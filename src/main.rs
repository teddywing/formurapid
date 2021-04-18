// Copyright (c) 2021  Teddy Wing
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


use anyhow;
use derive_builder::Builder;
use getopts::Options;
use pdf_forms::{Form, FieldType};
use serde::{Deserialize, Serialize};

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::process;

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

fn print_usage(opts: &Options) {
    print!(
        "{}",
        opts.usage("usage: formurapid [options] (--generate | --fill) PDF_FILE..."),
    );
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("error: {}", e);

            process::exit(exitcode::SOFTWARE);
        },
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("", "fill", "fill in the form using a markup file");
    opts.optflag("", "generate", "generate helper files to fill in the form");

    opts.optflag("h", "help", "print this help menu");
    opts.optflag("V", "version", "show the program version");

    let opt_matches = opts.parse(&args[1..])?;

    if opt_matches.opt_present("h") {
        print_usage(&opts);
        process::exit(exitcode::USAGE);
    }

    if opt_matches.opt_present("V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(exitcode::OK);
    }

    if opt_matches.free.is_empty() {
        print_usage(&opts);

        process::exit(exitcode::USAGE);
    }

    for pdf_path in &opt_matches.free {
        let form_path = Path::new(&pdf_path);
        let form_file_prefix = form_path
            .file_stem()
            .ok_or(anyhow::anyhow!("no file name"))?
            .to_str()
            .ok_or(anyhow::anyhow!("error converting file name"))?;

        let toml_file_name = format!("{}.toml", form_file_prefix);
        let toml_path = form_path.with_file_name(toml_file_name);
        let output_file_name = format!("{}-filled.pdf", form_file_prefix);

        let mut form = Form::load(form_path)?;

        if opt_matches.opt_present("fill") {
            fill(
                toml_path,
                form_path.with_file_name(output_file_name),
                &mut form,
            )?;
        } else if opt_matches.opt_present("generate") {
            let ids_path = form_path.with_file_name(
                format!("{}-ids.pdf", form_file_prefix)
            );

            generate_fill_helpers(
                toml_path,
                ids_path,
                &mut form,
            )?;
        } else {
            print_usage(&opts);

            process::exit(exitcode::USAGE);
        }
    }

    Ok(())
}

/// Generate files to fill in the form.
///
/// Generates a TOML file at `data_path`, and a PDF with IDs entered in the
/// form's text fields in `output_path`.
fn generate_fill_helpers<P: AsRef<Path>>(
    data_path: P,
    output_path: P,
    form: &mut Form,
) -> anyhow::Result<()> {
    let mut data = TextForm::default();

    for i in 0..form.len() {
        let field_type = form.get_type(i);

        match field_type {
            FieldType::Text => {
                form.set_text(i, format!("{}", i))?;

                data.fields.push(
                    FieldBuilder::default()
                        .id(i)
                        .value(Some(""))
                        .build()?
                );
            },
            FieldType::CheckBox => {
                data.fields.push(
                    FieldBuilder::default()
                        .id(i)
                        .state(Some(false))
                        .build()?
                );
            },
            _ => (),
        }
    }

    let mut toml_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(data_path)?;
    toml_file.write_all(&toml::to_vec(&data)?)?;

    form.save(output_path)?;

    Ok(())
}

/// Fill in `form` with values from `data_path` and write to `output_path`.
fn fill<P: AsRef<Path>>(
    data_path: P,
    output_path: P,
    form: &mut Form,
) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    let mut file = File::open(data_path)?;
    file.read_to_end(&mut buf)?;

    let data: TextForm = toml::from_slice(&buf)?;

    for field in data.fields {
        if let Some(value) = field.value {
            form.set_text(field.id, value.to_owned())?;
        } else if let Some(state) = field.state {
            form.set_check_box(field.id, state)?;
        }
    }

    form.save(output_path)?;

    Ok(())
}
