use crate::common::*;
use datamodel::{ast, diagnostics::DatamodelError};

const BLOB_TYPES: &[&'static str] = &["Blob", "LongBlob", "MediumBlob", "TinyBlob"];
const TEXT_TYPES: &[&'static str] = &["Text", "LongText", "MediumText", "TinyText"];

#[test]
fn text_and_blob_data_types_can_not_be_unique() {
    fn error_msg(type_name: &str) -> String {
        format!("Native type {} can not be unique in MySQL.", type_name)
    }

    for tpe in BLOB_TYPES {
        test_field_and_block_attribute_support(tpe, "Bytes", "unique", &error_msg(tpe));
    }

    for tpe in TEXT_TYPES {
        test_field_and_block_attribute_support(tpe, "String", "unique", &error_msg(tpe));
    }
}

#[test]
fn text_and_blob_data_types_should_fail_on_id_attribute() {
    fn error_msg(type_name: &str) -> String {
        format!(
            "Native type {} of MySQL can not be used on a field that is `@id` or `@@id`.",
            type_name
        )
    }

    for tpe in BLOB_TYPES {
        test_field_and_block_attribute_support(tpe, "Bytes", "id", &error_msg(tpe));
    }

    for tpe in TEXT_TYPES {
        test_field_and_block_attribute_support(tpe, "String", "id", &error_msg(tpe));
    }
}

fn test_field_and_block_attribute_support(native_type: &str, scalar_type: &str, attribute_name: &str, error_msg: &str) {
    let id_field = if attribute_name == "id" {
        ""
    } else {
        "id     Int    @id"
    };
    let dml = format!(
        r#"
    model Blog {{
      {id_field}
      bigInt {scalar_type} @db.{native_type} @{attribute_name}
    }}

    model User {{
      {id_field}
      firstname {scalar_type} @db.{native_type}
      lastname  {scalar_type} @db.{native_type}
      @@{attribute_name}([firstname, lastname])
    }}
    "#,
        id_field = id_field,
        native_type = native_type,
        scalar_type = scalar_type,
        attribute_name = attribute_name
    );

    test_compatibility(&dml, &error_msg);
}

fn test_compatibility(datamodel: &str, error_msg: &str) {
    let dml = format!(
        r#"
    datasource db {{
          provider = "mysql"
          url = "mysql://"
        }}

        generator js {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

    {datamodel}
    "#,
        datamodel = datamodel,
    );

    let error = parse_error(&dml);

    error.assert_length(2);
    error.assert_is_message_at(0, error_msg);
    error.assert_is_message_at(1, error_msg);
}

#[test]
fn should_fail_on_native_type_decimal_when_scale_is_bigger_than_precision() {
    let dml = r#"
        datasource db {
          provider = "mysql"
          url      = "mysql://"
        }

        generator js {
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int   @id
            dec Decimal @db.Decimal(2, 4)
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "The scale must not be larger than the precision for the Decimal native type in MySQL.",
        ast::Span::new(281, 311),
    ));
}

#[test]
fn should_fail_on_native_type_numeric_when_scale_is_bigger_than_precision() {
    let dml = r#"
        datasource db {
          provider = "mysql"
          url      = "mysql://"
        }

        generator js {
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int   @id
            dec Decimal @db.Numeric(2, 4)
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "The scale must not be larger than the precision for the Numeric native type in MySQL.",
        ast::Span::new(281, 311),
    ));
}

#[test]
fn should_fail_on_incompatible_scalar_type_with_tiny_int() {
    let dml = r#"
        datasource db {
          provider = "mysql"
          url = "mysql://"
        }

        generator js {
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            bigInt DateTime @db.TinyInt
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "Native type TinyInt is not compatible with declared field type DateTime, expected field type Boolean or Int.",
        ast::Span::new(294, 304),
    ));
}