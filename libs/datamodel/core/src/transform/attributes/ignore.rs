use super::{super::helpers::*, AttributeValidator};
use crate::diagnostics::DatamodelError;
use crate::Field::{RelationField, ScalarField};
use crate::Ignorable;
use crate::{ast, dml, Datamodel};

/// Prismas builtin `@ignore` attribute.
pub struct IgnoreAttributeValidator {}

const ATTRIBUTE_NAME: &str = "ignore";

impl AttributeValidator<dml::Model> for IgnoreAttributeValidator {
    fn attribute_name(&self) -> &str {
        ATTRIBUTE_NAME
    }

    fn validate_and_apply(&self, _args: &mut Arguments, obj: &mut dml::Model) -> Result<(), DatamodelError> {
        obj.is_ignored = true;
        Ok(())
    }

    fn serialize(&self, obj: &dml::Model, _datamodel: &Datamodel) -> Vec<ast::Attribute> {
        internal_serialize(obj)
    }
}

pub struct IgnoreAttributeValidatorForField {}
impl AttributeValidator<dml::Field> for IgnoreAttributeValidatorForField {
    fn attribute_name(&self) -> &str {
        ATTRIBUTE_NAME
    }

    fn validate_and_apply(&self, _args: &mut Arguments, obj: &mut dml::Field) -> Result<(), DatamodelError> {
        match obj {
            ScalarField(sf) => sf.is_ignored = true,
            RelationField(rf) => rf.is_ignored = true,
        }
        Ok(())
    }

    fn serialize(&self, obj: &dml::Field, _datamodel: &Datamodel) -> Vec<ast::Attribute> {
        internal_serialize(obj)
    }
}

fn internal_serialize(obj: &dyn Ignorable) -> Vec<ast::Attribute> {
    match obj.is_ignored() {
        true => vec![ast::Attribute::new(ATTRIBUTE_NAME, vec![])],
        false => vec![],
    }
}
