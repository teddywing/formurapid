use derive_builder::Builder;
use getopts::Options;
use pdf_forms::{Form, FieldType};
use serde::{Deserialize, Serialize};

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Default, Deserialize, Serialize)]
struct TextForm<'a> {
    #[serde(borrow)]
    fields: Vec<Field<'a>>,
}

#[derive(Debug, Builder, Deserialize, Serialize)]
struct Field<'a> {
    id: usize,

    #[builder(default)]
    value: Option<&'a str>,

    #[builder(default)]
    state: Option<bool>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("", "fill", "fill in the form using a markup file");

    let opt_matches = opts.parse(&args[1..]).unwrap();

    if opt_matches.free.is_empty() {
        return;
    }

    let form_path = &opt_matches.free[0];

    let mut form = Form::load(form_path).unwrap();

    if opt_matches.opt_present("fill") {
        fill("./f1040.toml", &mut form);

        return;
    }

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

fn fill<P: AsRef<Path>>(data_path: P, form: &mut Form) {
    let mut buf = Vec::new();
    let mut file = File::open(data_path).unwrap();
    file.read_to_end(&mut buf).unwrap();

    let data: TextForm = toml::from_slice(&buf).unwrap();

    for field in data.fields {
        if let Some(value) = field.value {
            form.set_text(field.id, value.to_owned()).unwrap();
        } else if let Some(state) = field.state {
            form.set_check_box(field.id, state).unwrap();
        }
    }

    form.save("./f1040-filled.pdf").unwrap();
}
