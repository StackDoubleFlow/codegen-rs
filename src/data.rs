use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum TypeEnum {
    Struct,
    Class,
    Enum,
    Interface,
}

#[derive(Deserialize, Debug)]
pub struct TypeDataThis {
    #[serde(rename = "Namespace")]
    pub namespace: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "QualifiedCppName")]
    pub qualified_cpp_name: String,
    #[serde(rename = "IsGenericTemplate")]
    pub is_generic_template: bool,
    #[serde(rename = "IsNested")]
    pub is_nested: bool,
    #[serde(rename = "ElementType")]
    pub element_type: Option<TypeRef>,
    #[serde(rename = "GenericParameterConstraints")]
    pub generic_parameter_constraints: Vec<TypeRef>,
    #[serde(rename = "Generics")]
    pub generics: Vec<TypeRef>,
}

#[derive(Deserialize, Debug)]
pub enum LayoutKind {
    Auto,
    Sequential,
    Explicit
}

#[derive(Deserialize, Debug)]
pub struct TypeData {
    #[serde(rename = "This")] 
    pub this: TypeDataThis,
    #[serde(rename = "Attributes")]
    pub attributes: Vec<Attribute>,
    #[serde(rename = "ImplementingInterfaces")]
    pub implementing_interfaces: Vec<TypeRef>,
    #[serde(rename = "InstanceFields")]
    pub instance_fields: Vec<Field>,
    #[serde(rename = "Layout")]
    pub layout: LayoutKind,
    #[serde(rename = "Methods")]
    pub methods: Vec<Method>,
    #[serde(rename = "NestedTypes")]
    pub nested_types: Vec<TypeData>,
    #[serde(rename = "Parent")]
    pub parent: Option<TypeRef>,
    #[serde(rename = "Properties")]
    pub properties: Vec<Property>,
    #[serde(rename = "Specifiers")]
    pub specifiers: Vec<String>,
    #[serde(rename = "StaticFields")]
    pub static_fields: Vec<Field>,
    #[serde(rename = "Type")]
    pub type_enum: TypeEnum,
    #[serde(rename = "TypeDefIndex")]
    pub type_def_index: i32,
    #[serde(rename = "Size")]
    pub size: i32,
}

#[derive(Deserialize, Debug)]
pub struct TypeRef {
    #[serde(rename = "Namespace")]
    pub namespace: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "TypeId")]
    pub type_id: i32,
    #[serde(rename = "Generics")]
    pub generics: Vec<TypeRef>,
}

#[derive(Deserialize, Debug)]
pub struct Method {
    #[serde(rename = "Attributes")]
    pub attributes: Vec<Attribute>,
    #[serde(rename = "Generic")]
    pub generic: bool,
    #[serde(rename = "GenericParameters")]
    pub generic_parameters: Vec<TypeRef>,
    #[serde(rename = "HidesBase")]
    pub hides_base: bool,
    #[serde(rename = "Il2CppName")]
    pub il2cpp_name: String,
    #[serde(rename = "ImplementedFrom")]
    pub implemented_from: Option<TypeRef>,
    #[serde(rename = "IsSpecialName")]
    pub is_special_name: bool,
    #[serde(rename = "IsVirtual")]
    pub is_virtual: bool,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Offset")]
    pub offset: i32,
    #[serde(rename = "Parameters")]
    pub parameters: Vec<Parameter>,
    #[serde(rename = "ReturnType")]
    pub return_type: TypeRef,
    #[serde(rename = "RVA")]
    pub rva: i32,
    #[serde(rename = "Slot")]
    pub slot: i32,
    #[serde(rename = "Specifiers")]
    pub specifiers: Vec<String>,
    #[serde(rename = "VA")]
    pub va: i32,
}

#[derive(Deserialize, Debug)]
pub struct Field {
    #[serde(rename = "Attributes")]
    pub attributes: Vec<Attribute>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Offset")]
    pub offset: i32,
    #[serde(rename = "LayoutOffset")]
    pub layout_offset: i32,
    #[serde(rename = "Specifiers")]
    pub specifiers: Vec<String>,
    #[serde(rename = "Type")]
    pub field_type: TypeRef,
}

#[derive(Deserialize, Debug)]
pub struct Attribute {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "RVA")]
    pub rva: i32,
    #[serde(rename = "Offset")]
    pub offset: i32,
    #[serde(rename = "VA")]
    pub va: i32,
}

// #[derive(Deserialize, Debug)]
// pub struct Specifier {
//     #[serde(rename = "Value")]
//     pub value: String,
// }

#[derive(Deserialize, Debug)]
pub enum ParameterModifier {
    None,
    Ref,
    Out,
    In,
    Params,
}

#[derive(Deserialize, Debug)]
pub struct Parameter {
    #[serde(rename = "Type")]
    pub parameter_type: TypeRef,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Modifier")]
    pub modifier: ParameterModifier,
}

#[derive(Deserialize, Debug)]
pub struct Property {
    #[serde(rename = "Attributes")]
    pub attributes: Vec<Attribute>,
    #[serde(rename = "Specifiers")]
    pub specifier: Vec<String>,
    #[serde(rename = "GetMethod")]
    pub get_method: bool,
    #[serde(rename = "SetMethod")]
    pub set_method: bool,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub property_type: TypeRef,
}

#[derive(Deserialize, Debug)]
pub struct DllData {
    #[serde(rename = "Types")]
    pub types: Vec<TypeData>,
}