use crate::core::model::model::Model;
use crate::parser::ast::argument::Argument;

pub(crate) fn before_delete_decorator(args: &Vec<Argument>, model: &mut Model) {
    model.set_before_delete_pipeline(args.get(0).unwrap().get_value().unwrap().as_pipeline().unwrap().clone());
}
