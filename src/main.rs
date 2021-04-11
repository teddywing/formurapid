use derive_builder::Builder;
use pdf_forms::{Form, FieldType};
use serde::Serialize;

use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Default, Serialize)]
struct TextForm<'a> {
    fields: Vec<Field<'a>>,
}

#[derive(Debug, Builder, Serialize)]
struct Field<'a> {
    id: usize,

    #[builder(default)]
    value: Option<&'a str>,

    #[builder(default)]
    state: Option<bool>,
}

fn main() {
    let mut form = Form::load("./f1040.pdf").unwrap();

    let mut data = TextForm::default();

    for i in 0..form.len() {
        let field_type = form.get_type(i);

        match field_type {
            FieldType::Text => {
                form.set_text(i, format!("{}", i)).unwrap();

                data.fields.push(FieldBuilder::default()
                    .id(i)
                    .value(Some(""))
                    .build()
                    .unwrap()
                );
            },
            FieldType::CheckBox => {
                data.fields.push(FieldBuilder::default()
                    .id(i)
                    .state(Some(false))
                    .build()
                    .unwrap()
                );
            },
            _ => (),
        }
    }

    let mut toml_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open("f1040.toml")
        .unwrap();
    toml_file.write_all(&toml::to_vec(&data).unwrap()).unwrap();

    form.save("./f1040-new.pdf").unwrap();
}
