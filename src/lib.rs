#![feature(plugin_registrar, box_syntax)]

#![feature(rustc_private)]

#[macro_use]
extern crate syntax;
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;

use rustc_plugin::Registry;

use rustc::hir::*;
use syntax::ast::{NodeId, Name};
use rustc::hir::map as ast_map;
use rustc::hir::intravisit as visit;
use rustc::traits::ProjectionMode;
use syntax::codemap::Span;
use rustc::lint::{LintPass, LintArray, LateLintPass, LintContext};
use rustc::lint::LateContext as Context;
use rustc::ty;
use rustc::middle::expr_use_visitor as euv;
use rustc::infer;
use rustc::middle::mem_categorization::{cmt, Categorization};
use syntax::attr::AttrMetaMethods;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box TenaciousPass);
}

#[allow(missing_copy_implementations)]
/// A lint pass which catches moves of types marked #[no_move]
pub struct TenaciousPass;

#[cfg(feature = "rvalue_checks")]
fn is_in_let(tcx: &ty::TyCtxt, id: NodeId) -> bool {
    if let ast_map::NodeStmt(ref st) = tcx.map.get(tcx.map.get_parent_node(id)) {
        if let StmtDecl(..) = st.node {
            println!("found");
            return true
        }
    }
    false
}

#[cfg(not(feature = "rvalue_checks"))]
fn is_in_let(_: &ty::TyCtxt, _: NodeId) -> bool {
    true
}

declare_lint!(pub MOVED_NO_MOVE, Warn,
              "warn on moving of immovable types");

impl LintPass for TenaciousPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(MOVED_NO_MOVE)
    }

}

impl LateLintPass for TenaciousPass {
    fn check_fn(&mut self, cx: &Context, _: visit::FnKind, decl: &FnDecl, body: &Block, _: Span, id: NodeId) {
        let param_env = ty::ParameterEnvironment::for_item(cx.tcx, id);
        let infcx = infer::new_infer_ctxt(cx.tcx, &cx.tcx.tables, Some(param_env), ProjectionMode::Any);
        let mut v = TenaciousDelegate(cx);
        let mut vis = euv::ExprUseVisitor::new(&mut v, &infcx);
        vis.walk_fn(decl, body)
    }
    fn check_struct_def(&mut self, cx: &Context, def: &VariantData, _: Name, _: &Generics, id: NodeId) {
        let item = match cx.tcx.map.get(id) {
            ast_map::NodeItem(it) => it,
            _ => cx.tcx.map.expect_item(cx.tcx.map.get_parent(id)),
        };
        if item.attrs.iter().all(|a| !a.check_name("no_move") && !a.check_name("allow_movable_interior")) {
            for ref field in def.fields() {
                if is_ty_no_move(cx.tcx, cx.tcx.node_id_to_type(field.id)) {
                    cx.span_lint(MOVED_NO_MOVE, field.span,
                                 "Structs and enums containing #[no_move] fields should be marked #[no_move]")
                }
            }
        }
    }
}


struct TenaciousDelegate<'a, 'tcx: 'a>(&'a Context<'a, 'tcx>);

impl<'a, 'tcx: 'a> euv::Delegate<'tcx> for TenaciousDelegate<'a, 'tcx> {
    fn consume(&mut self, _: NodeId, consume_span: Span,
               cmt: cmt<'tcx>, mode: euv::ConsumeMode) {
        if let Categorization::Rvalue(_) = cmt.cat {
            // Ignore `let x = rvalue()`
            if is_in_let(self.0.tcx, cmt.id) {
                return;
            }
        }
        if let euv::Move(..) = mode {
            if is_ty_no_move(self.0.tcx, cmt.ty) {
                self.0.span_lint(MOVED_NO_MOVE, consume_span,
                                 &format!("#[no_move] type `{:?}` moved", cmt.ty)[..])
            }
        }

    }
    fn matched_pat(&mut self, pat: &Pat, cmt: cmt<'tcx>, mode: euv::MatchMode) {
        if let Categorization::Rvalue(_) = cmt.cat {
            // Ignore `let x = ...`
            return;
        }
        if let euv::MovingMatch = mode {
            if is_ty_no_move(self.0.tcx, cmt.ty) {
                self.0.span_lint(MOVED_NO_MOVE, pat.span,
                                 &format!("#[no_move] type `{:?}` moved", cmt.ty)[..])
            }
        }
    }
    fn consume_pat(&mut self, pat: &Pat, cmt: cmt<'tcx>, mode: euv::ConsumeMode) {
        if let Categorization::Rvalue(_) = cmt.cat {
            // Ignore `let x = rvalue()`
            return;
        }
        if let euv::Move(_) = mode {
            if is_ty_no_move(self.0.tcx, cmt.ty) {
                self.0.span_lint(MOVED_NO_MOVE, pat.span,
                                 &format!("#[no_move] type `{:?}` moved", cmt.ty)[..])
            }
        }
    }
    fn borrow(&mut self, _: NodeId, _: Span, _: cmt<'tcx>, _: ty::Region,
              _: ty::BorrowKind, _: euv::LoanCause) {}
    fn decl_without_init(&mut self, _: NodeId, _: Span) {}
    fn mutate(&mut self, _: NodeId, _: Span, _: cmt<'tcx>, _: euv::MutateMode) {}
}

fn is_ty_no_move(tcx: &ty::TyCtxt, t: ty::Ty) -> bool {
    let mut found = false;
    t.maybe_walk(|ty| {
        match ty.sty {
            ty::TyStruct(did, _) | ty::TyEnum(did, _) => {
                if tcx.has_attr(did.did, "allow_movable_interior") {
                    return false;
                }
                if tcx.has_attr(did.did, "no_move") {
                    found = true;
                    return false;
                }
                if match_def_path(tcx, did.did, &["alloc", "rc", "Rc"]) ||
                   match_def_path(tcx, did.did, &["alloc", "arc", "Arc"]) {
                    return false;
                }
                true
            },
            ty::TyRef(..) | ty::TyRawPtr(..) | ty::TyBox(..) => false, // don't recurse down ptrs
            _ => true
        }
    });
    found
}

/// Check if a `DefId`'s path matches the given absolute type path usage.
///
/// # Examples
/// ```
/// match_def_path(cx, id, &["core", "option", "Option"])
/// ```
/// (Taken from clippy)
pub fn match_def_path(tcx: &ty::TyCtxt, def_id: def_id::DefId, path: &[&str]) -> bool {
    let krate = &tcx.crate_name(def_id.krate);
    if krate != &path[0] {
        return false;
    }

    let path = &path[1..];
    let other = tcx.def_path(def_id).data;

    if other.len() != path.len() {
        return false;
    }

    other.into_iter()
         .map(|e| e.data)
         .zip(path)
         .all(|(nm, p)| nm.as_interned_str() == *p)
}
