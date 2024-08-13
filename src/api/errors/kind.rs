use std::str::FromStr;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// access to the method is denied; in this case **reason** key is always present
    MethodForbidden,
    /// required parameter is not provided; an additional key **param_name** will contain the name of the parameter that caused this error;
    ParamMissing,
    /// value of the parameter is invalid; this type of error also contains **param_name** key;
    ParamInvalid,
    /// you are not allowed to use the parameter; in such case **param_name** and **reason** will be present;
    ParamForbidden,
    /// field specified in **fields** parameter does not exist; the name of the field will be passed in **field_name** key together with the name of the method (**method_name** key);
    FieldNotFound,
    /// specified field is invalid:
    /// - you have provided subfields for field that does not refer to subobject(s);
    /// - you have omitted subfields that were required;
    /// - you have used secondary field, but only primary were allowed.
    /// **field_name** and **method_name** keys will contain the name of the field and its method that caused the error.
    FieldInvalid,
    /// you do not have access to some of the requested fields (**field_name**, **method_name** and **reason** keys will be present);
    FieldForbidden,
    /// some of the referenced objects do not exist; if the object was referenced by one of parameters, the **param_name** and **method_name** keys will be present;
    ObjectNotFound,
    /// the referenced object is in state that prevents method execution; the detailed description of such errors is available in method documentation.
    ObjectInvalid,
    /// access to the referenced object was denied.
    ObjectForbidden,
}

impl FromStr for ErrorKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "method_forbidden" => Ok(Self::MethodForbidden),
            "param_missing" => Ok(Self::ParamMissing),
            "param_invalid" => Ok(Self::ParamInvalid),
            "param_forbidden" => Ok(Self::ParamForbidden),
            "field_not_found" => Ok(Self::FieldNotFound),
            "field_invalid" => Ok(Self::FieldInvalid),
            "field_forbidden" => Ok(Self::FieldForbidden),
            "object_not_found" => Ok(Self::ObjectNotFound),
            "object_invalid" => Ok(Self::ObjectInvalid),
            "object_forbidden" => Ok(Self::ObjectForbidden),
            _ => Err(String::from("Unexpected error kind")),
        }
    }
}
