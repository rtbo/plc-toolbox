use std::collections::HashSet;
use std::io::{BufRead, Write};
use std::{fs, io, path};

use quick_xml::Reader;
use quick_xml::events::{Event, attributes};

#[derive(Debug)]
struct EnumValue {
    name: String,
    value: u32,
}

#[derive(Debug)]
enum DataType {
    Structure {
        name: String,
        doc: Option<String>,
    },
    Enumeration {
        name: String,
        length_in_bits: u32,
        doc: Option<String>,
        values: Vec<EnumValue>,
    },
    Opaque {
        name: String,
        doc: Option<String>,
    },
}

impl DataType {
    fn name(&self) -> &str {
        match self {
            DataType::Structure { name, .. } => name,
            DataType::Enumeration { name, .. } => name,
            DataType::Opaque { name, .. } => name,
        }
    }

    fn doc(&self) -> Option<&str> {
        match self {
            DataType::Structure { doc, .. } => doc.as_deref(),
            DataType::Enumeration { doc, .. } => doc.as_deref(),
            DataType::Opaque { doc, .. } => doc.as_deref(),
        }
    }
}

fn parse_data_types(bsd_path: &path::Path) -> Vec<DataType> {
    let mut reader = Reader::from_file(bsd_path).unwrap();
    reader.config_mut().trim_text(true);

    let mut data_types = Vec::new();
    let mut buf = Vec::new();
    let mut buf2 = Vec::new();

    loop {
        let ev = reader.read_event_into(&mut buf);
        match ev {
            Ok(Event::Eof) => break data_types,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                "opc:StructuredType" => {
                    let data_type = parse_structured_type(&mut reader, &mut buf2, e.attributes());
                    data_types.push(data_type);
                }
                "opc:EnumeratedType" => {
                    let data_type = parse_enumerated_type(&mut reader, &mut buf2, e.attributes());
                    data_types.push(data_type);
                }
                "opc:OpaqueType" => {
                    let data_type = parse_opaque_type(&mut reader, &mut buf2, e.attributes());
                    data_types.push(data_type);
                }
                _ => (),
            },
            Ok(_) => (),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }
}

fn parse_structured_type<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    mut attributes: attributes::Attributes,
) -> DataType {
    let name = attributes
        .find(|attr| {
            attr.as_ref()
                .map(|a| a.key.as_ref() == "Name")
                .unwrap_or(false)
        })
        .map(|attr| attr.unwrap())
        .map(|attr| {
            attr.normalized_value(Default::default())
                .unwrap()
                .into_owned()
        })
        .unwrap();

    let mut doc = None;
    let mut in_doc = false;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::End(e)) if e.name().as_ref() == "opc:StructuredType" => break,
            Ok(Event::Start(e)) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Text(e)) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            Ok(_) => (),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    DataType::Structure { name, doc }
}

fn parse_enumerated_type<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    mut attributes: attributes::Attributes,
) -> DataType {
    let name = attributes
        .find(|attr| {
            attr.as_ref()
                .map(|a| a.key.as_ref() == "Name")
                .unwrap_or(false)
        })
        .map(|attr| attr.unwrap())
        .map(|attr| {
            attr.normalized_value(Default::default())
                .unwrap()
                .into_owned()
        })
        .unwrap();
    let length_in_bits = attributes
        .find(|attr| {
            attr.as_ref()
                .map(|a| a.key.as_ref() == "LengthInBits")
                .unwrap_or(false)
        })
        .map(|attr| attr.unwrap())
        .map(|attr| {
            attr.normalized_value(Default::default())
                .unwrap()
                .into_owned()
                .parse::<u32>()
                .expect("Failed to parse LengthInBits")
        })
        .unwrap();

    let mut values = Vec::new();
    let mut doc = None;
    let mut in_doc = false;
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Ok(Event::End(e)) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = false;
            }
            Ok(Event::Start(e) | Event::Empty(e)) if e.name().as_ref() == "opc:EnumeratedValue" => {
                let name = e
                    .try_get_attribute("Name")
                    .unwrap()
                    .expect("Missing Name attribute")
                    .normalized_value(Default::default())
                    .unwrap()
                    .into_owned();
                let value = e
                    .try_get_attribute("Value")
                    .unwrap()
                    .expect("Missing Value attribute")
                    .normalized_value(Default::default())
                    .unwrap()
                    .parse()
                    .expect("Failed to parse Value");
                values.push(EnumValue { name, value });
            }
            Ok(Event::Text(e)) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            Ok(Event::End(e)) if e.name().as_ref() == "opc:EnumeratedType" => break,
            Ok(Event::Eof) => break,
            Ok(_) => (),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    DataType::Enumeration {
        name,
        length_in_bits,
        doc,
        values,
    }
}

fn parse_opaque_type<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    mut attributes: attributes::Attributes,
) -> DataType {
    let name = attributes
        .find(|attr| {
            attr.as_ref()
                .map(|a| a.key.as_ref() == "Name")
                .unwrap_or(false)
        })
        .map(|attr| attr.unwrap())
        .map(|attr| {
            attr.normalized_value(Default::default())
                .unwrap()
                .into_owned()
        })
        .unwrap();

    let mut doc = None;
    let mut in_doc = false;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::End(e)) if e.name().as_ref() == "opc:OpaqueType" => break,
            Ok(Event::Start(e)) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Text(e)) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            Ok(_) => (),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    DataType::Opaque { name, doc }
}

fn read_types_set(types_list_path: &path::Path) -> HashSet<String> {
    let reader = io::BufReader::new(fs::File::open(&types_list_path).unwrap());

    let mut types_set = HashSet::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        types_set.insert(line.to_string());
    }
    types_set
}

fn generate_type_rs<W: io::Write>(out: &mut W, typ: &DataType) -> io::Result<()> {
    if let Some(doc) = typ.doc() {
        let lines = doc.lines();
        for line in lines {
            writeln!(out, "/// {}", line)?;
        }
    }
    match typ {
        DataType::Structure { name, .. } | DataType::Opaque { name, .. } => {
            writeln!(out, "#[repr(transparent)]")?;
            writeln!(out, "pub struct {} {{", name)?;
            writeln!(out, "    raw: crate::ffi::UA_{},", name)?;
            writeln!(out, "}}")?;
            writeln!(out)?;
            writeln!(out, "unsafe impl crate::DataType for {} {{", name)?;
            writeln!(out, "    type Raw = crate::ffi::UA_{};", name)?;
            writeln!(
                out,
                "    const UA_TYPE_IDX: usize = crate::ffi::UA_TYPES_{} as usize;",
                name.to_uppercase()
            )?;
            writeln!(out, "    const NAME: &'static str = \"{}\";", name)?;
            writeln!(out)?;
            writeln!(out, "    /// Constructs an instance from the raw value.")?;
            writeln!(out, "    ///")?;
            writeln!(out, "    /// # Safety")?;
            writeln!(out, "    ///")?;
            writeln!(
                out,
                "    /// The caller must ensure that the raw value is valid."
            )?;
            writeln!(out, "    unsafe fn from_raw(raw: Self::Raw) -> Self {{")?;
            writeln!(out, "        Self {{ raw }}")?;
            writeln!(out, "    }}")?;
            writeln!(out, "    fn into_raw(self) -> Self::Raw {{")?;
            writeln!(out, "        // SAFETY: The raw value is always valid.")?;
            writeln!(
                out,
                "        let raw = unsafe {{ std::ptr::read(&self.raw) }};"
            )?;
            writeln!(out, "        std::mem::forget(self);")?;
            writeln!(out, "        raw")?;
            writeln!(out, "    }}")?;
            writeln!(out, "    fn as_raw(&self) -> &Self::Raw {{")?;
            writeln!(out, "        &self.raw")?;
            writeln!(out, "    }}")?;
            writeln!(out, "    fn as_raw_mut(&mut self) -> &mut Self::Raw {{")?;
            writeln!(out, "        &mut self.raw")?;
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;
        }
        DataType::Enumeration {
            name,
            length_in_bits,
            values,
            ..
        } => {
            assert!(
                length_in_bits % 8 == 0,
                "{}: Length in bits is not a multiple of 8",
                name,
            );
            writeln!(out, "#[repr(u{})]", length_in_bits)?;
            writeln!(out, "pub enum {} {{", name)?;
            for value in values {
                writeln!(out, "    {} = {},", value.name, value.value)?;
            }
            writeln!(out, "}}")?;
        }
    }
    Ok(())
}

pub fn generate_rs(schema_dir: &path::Path, out_dir: &path::Path) {
    let types_list_path = schema_dir.join("datatypes.txt");
    let bsd_path = schema_dir.join("Opc.Ua.Types.bsd");
    println!("cargo:rerun-if-changed={}", types_list_path.display());
    println!("cargo:rerun-if-changed={}", bsd_path.display());

    let data_types = parse_data_types(&bsd_path);

    let type_list = { read_types_set(&types_list_path) };

    let out_path = out_dir.join("data_types.rs");
    let out_file = fs::File::create(&out_path).unwrap();
    let mut output = io::BufWriter::new(out_file);

    for (i, typ) in data_types
        .iter()
        .filter(|typ| type_list.contains(typ.name()))
        .enumerate()
    {
        if i > 0 {
            writeln!(&mut output).unwrap();
        }
        generate_type_rs(&mut output, typ).unwrap();
    }
}
