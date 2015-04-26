#![feature(plugin_registrar, box_syntax)]

#![feature(rustc_private)]

#[macro_use]
extern crate syntax;
#[macro_use]
extern crate rustc;

use rustc::plugin::Registry;
use rustc::lint::LintPassObject;

use syntax::ast::*;
use syntax::visit;
use syntax::codemap::Span;
use rustc::lint::{Context, LintPass, LintArray};
use rustc::middle::ty;
use rustc::util::ppaux::Repr;
use rustc::middle::expr_use_visitor as euv;
use rustc::middle::mem_categorization::{cmt, categorization};
use syntax::attr::AttrMetaMethods;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_lint_pass(box TenaciousPass as LintPassObject);
}

#[allow(missing_copy_implementations)]
/// A lint pass which catches moves of types marked #[no_move]
pub struct TenaciousPass;


declare_lint!(pub MOVED_NO_MOVE, Warn,
              "warn on moving of immovable types");

impl LintPass for TenaciousPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(MOVED_NO_MOVE)
    }

    fn check_fn(&mut self, cx: &Context, _: visit::FnKind, decl: &FnDecl, body: &Block, _: Span, id: NodeId) {
        let param_env = ty::ParameterEnvironment::for_item(cx.tcx, id);
        let mut v = TenaciousDelegate(cx);
        let mut vis = euv::ExprUseVisitor::new(&mut v, &param_env);
        vis.walk_fn(decl, body)
    }
    fn check_struct_def(&mut self, cx: &Context, def: &StructDef, _: Ident, _: &Generics, id: NodeId) {
        if cx.tcx.map.expect_item(id).attrs.iter().all(|a| !a.check_name("no_move")) {
            for ref field in def.fields.iter() {
                if is_ty_no_move(cx.tcx, ty::node_id_to_type(cx.tcx, field.node.id)) {
                    cx.span_lint(MOVED_NO_MOVE, field.span,
                                 "Structs containing #[no_move] fields should be marked #[no_move]")
                }
            }
        }
    }
    fn check_variant(&mut self, cx: &Context, var: &Variant, _: &Generics) {
        let ref map = cx.tcx.map;
        if map.expect_item(map.get_parent(var.node.id)).attrs.iter().all(|a| !a.check_name("no_move")) {
            match var.node.kind {
                TupleVariantKind(_) => {
                    if is_ty_no_move(cx.tcx, ty::node_id_to_type(cx.tcx, var.node.id)) {
                        cx.span_lint(MOVED_NO_MOVE, var.span,
                                     "Enums containing #[no_move] fields should be marked #[no_move]")
                    }
                }
                _ => () // Struct variants already caught by check_struct_def
            }
        }
    }
        
}


struct TenaciousDelegate<'a, 'tcx: 'a>(&'a Context<'a, 'tcx>);

impl<'a, 'tcx: 'a> euv::Delegate<'tcx> for TenaciousDelegate<'a, 'tcx> {
    fn consume(&mut self, _: NodeId, consume_span: Span,
               cmt: cmt<'tcx>, mode: euv::ConsumeMode) {
        if let euv::Move(_) = mode {
            if is_ty_no_move(self.0.tcx, cmt.ty) {
                self.0.span_lint(MOVED_NO_MOVE, consume_span,
                                 &format!("#[no_move] type `{}` moved", cmt.ty.repr(self.0.tcx))[..])
            }
        }

    }
    fn matched_pat(&mut self, pat: &Pat, cmt: cmt<'tcx>, mode: euv::MatchMode) {
        if let euv::MovingMatch = mode {
            if is_ty_no_move(self.0.tcx, cmt.ty) {
                self.0.span_lint(MOVED_NO_MOVE, pat.span,
                                 &format!("#[no_move] type `{}` moved", cmt.ty.repr(self.0.tcx))[..])
            }
        }
    }
    fn consume_pat(&mut self, pat: &Pat, cmt: cmt<'tcx>, mode: euv::ConsumeMode) {
        if let categorization::cat_rvalue(_) = cmt.cat {
            // Ignore `let x = rvalue()`
            return;
        }
        if let euv::Move(_) = mode {
            if is_ty_no_move(self.0.tcx, cmt.ty) {
                self.0.span_lint(MOVED_NO_MOVE, pat.span,
                                 &format!("#[no_move] type `{}` moved", cmt.ty.repr(self.0.tcx))[..])
            }
        }
    }
    fn borrow(&mut self, _: NodeId, _: Span, _: cmt<'tcx>, _: ty::Region,
              _: ty::BorrowKind, _: euv::LoanCause) {}
    fn decl_without_init(&mut self, _: NodeId, _: Span) {}
    fn mutate(&mut self, _: NodeId, _: Span, _: cmt<'tcx>, _: euv::MutateMode) {}
}

fn is_ty_no_move(tcx: &ty::ctxt, t: ty::Ty) -> bool {
    let mut found = false;
    ty::maybe_walk_ty(t, |ty| {
        match ty.sty {
            ty::ty_struct(did, _) | ty::ty_enum(did, _) => {
                if ty::has_attr(tcx, did, "no_move") {
                    found = true;
                }
                true
            },
            ty::ty_ptr(..) | ty::ty_rptr(..) => false, // don't recurse down ptrs
            _ => true
        }
    });
    found
}