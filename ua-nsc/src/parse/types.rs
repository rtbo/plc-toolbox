use std::collections::HashMap;
use std::sync::Arc;
/// OPC Binary Schema definition (BSD) types and parsing utilities.
use std::{fmt, path};
use std::{fs, io};

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

#[derive(Clone, Debug)]
pub enum Error {
    Io(Arc<io::Error>),
    Xml(quick_xml::Error),
    InvalidValue { typ: String, value: String },
    MissingRequiredAttribute(String),
    UnexpectedEof(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Xml(err) => write!(f, "XML error: {}", err),
            Error::InvalidValue { typ, value } => write!(f, "Invalid value for {}: {}", typ, value),
            Error::MissingRequiredAttribute(attr) => {
                write!(f, "Missing required attribute: {}", attr)
            }
            Error::UnexpectedEof(tag) => write!(f, "Unexpected end of file while parsing {}", tag),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(Arc::new(err))
    }
}

impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        match err {
            quick_xml::Error::Io(err) => Error::Io(err),
            _ => Error::Xml(err),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct ImportDirective {
    pub namespace: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

impl std::str::FromStr for ByteOrder {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "BigEndian" => Ok(ByteOrder::BigEndian),
            "LittleEndian" => Ok(ByteOrder::LittleEndian),
            _ => Err(Error::InvalidValue {
                typ: "ByteOrder".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchOperand {
    Equals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    NotEqual,
}

impl std::str::FromStr for SwitchOperand {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Equals" => Ok(SwitchOperand::Equals),
            "GreaterThan" => Ok(SwitchOperand::GreaterThan),
            "LessThan" => Ok(SwitchOperand::LessThan),
            "GreaterThanOrEqual" => Ok(SwitchOperand::GreaterThanOrEqual),
            "LessThanOrEqual" => Ok(SwitchOperand::LessThanOrEqual),
            "NotEqual" => Ok(SwitchOperand::NotEqual),
            _ => Err(Error::InvalidValue {
                typ: "SwitchOperand".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

/// Common attributes/content shared by OpaqueType, EnumeratedType and StructuredType (xs:TypeDescription).
#[derive(Debug, Clone, Default)]
pub struct TypeDescription {
    pub name: String,
    pub default_byte_order: Option<ByteOrder>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: Option<String>,
    pub value: Option<u64>,
    pub doc: Option<String>,
}

/// Qualified name (`prefix:name` in the BSD), with the prefix resolved to its namespace URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualName {
    pub namespace: Option<String>,
    pub ns_shorthand: Option<String>,
    pub name: String,
}

impl fmt::Display for QualName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.ns_shorthand {
            Some(ns) => write!(f, "{}:{}", ns, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

/// Maps every `xmlns` prefix declared on the dictionary (the default one under `""`) to its URI.
type Prefixes = HashMap<String, String>;

impl QualName {
    fn parse(value: &str, prefixes: &Prefixes) -> Result<Self> {
        let (ns_shorthand, name) = match value.split_once(':') {
            Some((prefix, name)) => (Some(prefix.to_string()), name),
            None => (None, value),
        };
        let namespace = match &ns_shorthand {
            Some(prefix) => {
                Some(
                    prefixes
                        .get(prefix)
                        .cloned()
                        .ok_or_else(|| Error::InvalidValue {
                            typ: "QName".to_string(),
                            value: value.to_string(),
                        })?,
                )
            }
            None => prefixes.get("").cloned(),
        };
        Ok(QualName {
            namespace,
            ns_shorthand,
            name: name.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub type_name: Option<QualName>,
    pub length: Option<u32>,
    pub length_field: Option<String>,
    pub is_length_in_bytes: bool,
    pub switch_field: Option<String>,
    pub switch_value: Option<u32>,
    pub switch_operand: Option<SwitchOperand>,
    pub terminator: Option<Vec<u8>>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Opaque {
        desc: TypeDescription,
        length_in_bits: Option<u32>,
        byte_order_significant: bool,
    },
    Enumerated {
        desc: TypeDescription,
        length_in_bits: Option<u32>,
        byte_order_significant: bool,
        is_option_set: bool,
        values: Vec<EnumValue>,
    },
    Structured {
        base_type: Option<QualName>,
        desc: TypeDescription,
        fields: Vec<Field>,
    },
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Type::Opaque { desc, .. } => &desc.name,
            Type::Enumerated { desc, .. } => &desc.name,
            Type::Structured { desc, .. } => &desc.name,
        }
    }

    pub fn doc(&self) -> Option<&str> {
        match self {
            Type::Opaque { desc, .. } => desc.doc.as_deref(),
            Type::Enumerated { desc, .. } => desc.doc.as_deref(),
            Type::Structured { desc, .. } => desc.doc.as_deref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeDictionary {
    pub target_namespace: String,
    /// `(namespace URI, short name)` pairs in declaration order, duplicates included
    /// (future module names in generated code).
    pub ns_table: Vec<(String, String)>,
    pub default_byte_order: Option<ByteOrder>,
    pub doc: Option<String>,
    pub imports: Vec<ImportDirective>,
    pub types: Vec<Type>,
}

/// Reads an optional attribute's normalized value as a `String`.
fn get_attr(e: &BytesStart, name: &str) -> Option<String> {
    e.try_get_attribute(name)
        .unwrap_or_else(|err| panic!("Invalid attribute {name}: {err:?}"))
        .map(|attr| {
            attr.normalized_value(Default::default())
                .unwrap()
                .into_owned()
        })
}

fn get_attr_required(e: &BytesStart, name: &str) -> Result<String> {
    get_attr(e, name).ok_or_else(|| Error::MissingRequiredAttribute(name.to_string()))
}

fn get_attr_u32(e: &BytesStart, name: &str) -> Result<Option<u32>> {
    get_attr(e, name)
        .map(|v| v.parse())
        .transpose()
        .map_err(|_| Error::InvalidValue {
            typ: "u32".to_string(),
            value: get_attr(e, name).unwrap_or_default(),
        })
}

fn get_attr_u64(e: &BytesStart, name: &str) -> Result<Option<u64>> {
    get_attr(e, name)
        .map(|v| v.parse())
        .transpose()
        .map_err(|_| Error::InvalidValue {
            typ: "u64".to_string(),
            value: get_attr(e, name).unwrap_or_default(),
        })
}

fn get_attr_bool(e: &BytesStart, name: &str, default: bool) -> Result<bool> {
    Ok(get_attr(e, name)
        .map(|v| match v.as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            other => Err(Error::InvalidValue {
                typ: "bool".to_string(),
                value: other.to_string(),
            }),
        })
        .transpose()?
        .unwrap_or(default))
}

fn get_attr_byte_order(e: &BytesStart, name: &str) -> Result<Option<ByteOrder>> {
    get_attr(e, name).map(|v| v.parse()).transpose()
}

fn get_attr_switch_operand(e: &BytesStart, name: &str) -> Result<Option<SwitchOperand>> {
    get_attr(e, name).map(|v| v.parse()).transpose()
}

fn get_attr_hex_binary(e: &BytesStart, name: &str) -> Option<Vec<u8>> {
    get_attr(e, name).map(|v| {
        (0..v.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&v[i..i + 2], 16)
                    .unwrap_or_else(|err| panic!("Failed to parse {name} as hexBinary: {err:?}"))
            })
            .collect()
    })
}

fn parse_type_description(e: &BytesStart) -> Result<TypeDescription> {
    Ok(TypeDescription {
        name: get_attr_required(e, "Name")?,
        default_byte_order: get_attr_byte_order(e, "DefaultByteOrder")?,
        doc: None,
    })
}

const BSD_NAMESPACE: &str = "http://opcfoundation.org/BinarySchema/";
const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

fn parse_prefixes(e: &BytesStart) -> Result<Prefixes> {
    let mut prefixes = Prefixes::new();
    for attr in e.attributes() {
        let attr = attr.map_err(quick_xml::Error::from)?;
        let key = attr.key.as_ref();
        let prefix = if key == "xmlns" {
            ""
        } else if let Some(prefix) = key.strip_prefix("xmlns:") {
            prefix
        } else {
            continue;
        };
        let uri = attr.normalized_value(Default::default())?.into_owned();
        prefixes.entry(prefix.to_string()).or_insert(uri);
    }
    Ok(prefixes)
}

/// Builds the namespace table from the `xmlns:*` declarations of a tag.
/// The schema infrastructure namespaces are skipped; several prefixes
/// declaring the same namespace are all kept.
fn parse_ns_table(e: &BytesStart) -> Result<Vec<(String, String)>> {
    let mut table = Vec::new();
    for attr in e.attributes() {
        let attr = attr.map_err(quick_xml::Error::from)?;
        let Some(prefix) = attr.key.as_ref().strip_prefix("xmlns:") else {
            continue;
        };
        let prefix = prefix.to_string();
        let uri = attr.normalized_value(Default::default())?.into_owned();
        if uri == BSD_NAMESPACE || uri == XSI_NAMESPACE {
            continue;
        }
        table.push((uri, prefix));
    }
    Ok(table)
}

/// Parses a BSD file. `ns_overrides` maps namespace URIs to short names and
/// are stored as-is in the dictionary and do not alter `ns_table`.
pub fn parse_types_bsd(bsd_path: &path::Path) -> Result<TypeDictionary> {
    let file = fs::File::open(bsd_path)?;
    let file = io::BufReader::new(file);
    let mut reader = Reader::from_reader(file);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => return Err(Error::UnexpectedEof("opc:TypeDictionary".to_string())),
            Event::Start(e) if e.name().as_ref() == "opc:TypeDictionary" => {
                let target_namespace = get_attr_required(&e, "TargetNamespace")?;
                let default_byte_order = get_attr_byte_order(&e, "DefaultByteOrder")?;
                let ns_table = parse_ns_table(&e)?;
                let prefixes = parse_prefixes(&e)?;
                return parse_type_dictionary(
                    &mut reader,
                    &mut buf,
                    &prefixes,
                    target_namespace,
                    ns_table,
                    default_byte_order,
                );
            }
            _ => (),
        }
    }
}

fn parse_import_directive(e: &BytesStart) -> ImportDirective {
    ImportDirective {
        namespace: get_attr(e, "Namespace"),
        location: get_attr(e, "Location"),
    }
}

fn parse_type_dictionary<R>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    prefixes: &Prefixes,
    target_namespace: String,
    ns_table: Vec<(String, String)>,
    default_byte_order: Option<ByteOrder>,
) -> Result<TypeDictionary>
where
    R: io::BufRead + Unpin,
{
    let mut doc = None;
    let mut in_doc = false;
    let mut imports = Vec::new();
    let mut types = Vec::new();
    let mut buf2 = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::End(e) if e.name().as_ref() == "opc:TypeDictionary" => break,
            Event::Start(e) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Event::End(e) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = false;
            }
            Event::Empty(e) | Event::Start(e) if e.name().as_ref() == "opc:Import" => {
                imports.push(parse_import_directive(&e));
            }
            Event::Start(e) if e.name().as_ref() == "opc:StructuredType" => {
                types.push(parse_structured_type(reader, &mut buf2, &e, prefixes)?);
            }
            Event::Start(e) if e.name().as_ref() == "opc:EnumeratedType" => {
                types.push(parse_enumerated_type(reader, &mut buf2, &e)?);
            }
            Event::Start(e) if e.name().as_ref() == "opc:OpaqueType" => {
                types.push(parse_opaque_type(reader, &mut buf2, &e)?);
            }
            Event::Text(e) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            Event::Eof => break,
            _ => (),
        }
    }

    Ok(TypeDictionary {
        target_namespace,
        ns_table,
        default_byte_order,
        doc,
        imports,
        types,
    })
}

/// Reads the optional `Documentation` element's text content, returning when `end_tag` closes.
fn read_doc_and_skip_to_end<R>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    end_tag: &str,
) -> Result<Option<String>>
where
    R: io::BufRead + Unpin,
{
    let mut doc = None;
    let mut in_doc = false;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Event::End(e) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = false;
            }
            Event::Text(e) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            Event::End(e) if e.name().as_ref() == end_tag => break,
            Event::Eof => break,
            _ => (),
        }
    }

    Ok(doc)
}

fn parse_field(e: &BytesStart, prefixes: &Prefixes) -> Result<Field> {
    Ok(Field {
        name: get_attr_required(e, "Name")?,
        type_name: get_attr(e, "TypeName")
            .map(|v| QualName::parse(&v, prefixes))
            .transpose()?,
        length: get_attr_u32(e, "Length")?,
        length_field: get_attr(e, "LengthField"),
        is_length_in_bytes: get_attr_bool(e, "IsLengthInBytes", false)?,
        switch_field: get_attr(e, "SwitchField"),
        switch_value: get_attr_u32(e, "SwitchValue")?,
        switch_operand: get_attr_switch_operand(e, "SwitchOperand")?,
        terminator: get_attr_hex_binary(e, "Terminator"),
        doc: None,
    })
}

fn parse_structured_type<R>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
    prefixes: &Prefixes,
) -> Result<Type>
where
    R: io::BufRead + Unpin,
{
    let desc = parse_type_description(start)?;
    let base_type = get_attr(start, "BaseType")
        .map(|v| QualName::parse(&v, prefixes))
        .transpose()?;

    let mut doc = None;
    let mut in_doc = false;
    let mut fields = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::End(e)) if e.name().as_ref() == "opc:StructuredType" => break,
            Ok(Event::Start(e)) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Ok(Event::End(e)) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = false;
            }
            Ok(Event::Start(e)) if e.name().as_ref() == "opc:Field" => {
                let mut field = parse_field(&e, prefixes)?;
                field.doc = read_doc_and_skip_to_end(reader, buf, "opc:Field")?;
                fields.push(field);
            }
            Ok(Event::Empty(e)) if e.name().as_ref() == "opc:Field" => {
                fields.push(parse_field(&e, prefixes)?);
            }
            Ok(Event::Eof) => break,
            Ok(Event::Text(e)) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            Ok(_) => (),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    Ok(Type::Structured {
        desc: TypeDescription { doc, ..desc },
        base_type,
        fields,
    })
}

fn parse_enumerated_value<R>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<String>,
    value: Option<u64>,
) -> Result<EnumValue>
where
    R: io::BufRead + Unpin,
{
    Ok(EnumValue {
        name,
        value,
        doc: read_doc_and_skip_to_end(reader, buf, "opc:EnumeratedValue")?,
    })
}

fn parse_enumerated_type<R: io::BufRead + Unpin>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<Type> {
    let desc = parse_type_description(start)?;
    let length_in_bits = get_attr_u32(start, "LengthInBits")?;
    let byte_order_significant = get_attr_bool(start, "ByteOrderSignificant", false)?;
    let is_option_set = get_attr_bool(start, "IsOptionSet", false)?;

    let mut values = Vec::new();
    let mut doc = None;
    let mut in_doc = false;
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Event::End(e) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = false;
            }
            Event::Empty(e) if e.name().as_ref() == "opc:EnumeratedValue" => {
                values.push(EnumValue {
                    name: get_attr(&e, "Name"),
                    value: get_attr_u64(&e, "Value")?,
                    doc: None,
                });
            }
            Event::Start(e) if e.name().as_ref() == "opc:EnumeratedValue" => {
                let name = get_attr(&e, "Name");
                let value_attr = get_attr_u64(&e, "Value")?;
                let value = parse_enumerated_value(reader, buf, name, value_attr)?;
                values.push(value);
            }
            Event::Text(e) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            Event::End(e) if e.name().as_ref() == "opc:EnumeratedType" => break,
            Event::Eof => break,
            _ => (),
        }
    }

    Ok(Type::Enumerated {
        desc: TypeDescription { doc, ..desc },
        length_in_bits,
        byte_order_significant,
        is_option_set,
        values,
    })
}

fn parse_opaque_type<R: io::BufRead + Unpin>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<Type> {
    let desc = parse_type_description(start)?;
    let length_in_bits = get_attr_u32(start, "LengthInBits")?;
    let byte_order_significant = get_attr_bool(start, "ByteOrderSignificant", false)?;

    let mut doc = None;
    let mut in_doc = false;

    loop {
        match reader.read_event_into(buf)? {
            Event::End(e) if e.name().as_ref() == "opc:OpaqueType" => break,
            Event::Start(e) if e.name().as_ref() == "opc:Documentation" => {
                in_doc = true;
            }
            Event::Eof => break,
            Event::Text(e) if in_doc => {
                doc = Some(e.into_inner().into_owned());
            }
            _ => (),
        }
    }

    Ok(Type::Opaque {
        desc: TypeDescription { doc, ..desc },
        length_in_bits,
        byte_order_significant,
    })
}
