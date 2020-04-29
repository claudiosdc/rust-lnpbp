// LNP/BP Rust Library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

impl FieldFormat {
    pub fn validate(&self, value: &Value) -> Result<(), SchemaError> {
        match (self, value) {
            (
                Self::Unsigned {
                    bits: Bits::Bit256,
                    min: None,
                    max: None,
                },
                Value::U256(_),
            ) => Ok(()),
            (
                Self::Unsigned {
                    bits: Bits::Bit256, ..
                },
                Value::U256(_),
            ) => Err(SchemaError::MinMaxBoundsOnLargeInt),
            (
                Self::Unsigned {
                    bits: Bits::Bit128,
                    min: None,
                    max: None,
                },
                Value::U128(_),
            ) => Ok(()),
            (
                Self::Unsigned {
                    bits: Bits::Bit128, ..
                },
                Value::U128(_),
            ) => Err(SchemaError::MinMaxBoundsOnLargeInt),
            (
                Self::Unsigned {
                    bits: Bits::Bit64,
                    min,
                    max,
                },
                Value::U64(val),
            ) if *val >= min.unwrap_or(0) && *val <= max.unwrap_or(u64::MAX) => Ok(()),
            (
                Self::Unsigned {
                    bits: Bits::Bit32,
                    min,
                    max,
                },
                Value::U32(val),
            ) if *val as u64 >= min.unwrap_or(0)
                && *val as u64 <= max.unwrap_or(u32::MAX as u64) =>
            {
                Ok(())
            }
            (
                Self::Unsigned {
                    bits: Bits::Bit16,
                    min,
                    max,
                },
                Value::U16(val),
            ) if *val as u64 >= min.unwrap_or(0)
                && *val as u64 <= max.unwrap_or(u16::MAX as u64) =>
            {
                Ok(())
            }
            (
                Self::Unsigned {
                    bits: Bits::Bit8,
                    min,
                    max,
                },
                Value::U8(val),
            ) if *val as u64 >= min.unwrap_or(0)
                && *val as u64 <= max.unwrap_or(u8::MAX as u64) =>
            {
                Ok(())
            }
            (
                Self::Integer {
                    bits: Bits::Bit64,
                    min,
                    max,
                },
                Value::I64(val),
            ) if *val >= min.unwrap_or(0) && *val <= max.unwrap_or(i64::MAX) => Ok(()),
            (
                Self::Integer {
                    bits: Bits::Bit32,
                    min,
                    max,
                },
                Value::I32(val),
            ) if *val as i64 >= min.unwrap_or(0)
                && *val as i64 <= max.unwrap_or(i32::MAX as i64) =>
            {
                Ok(())
            }
            (
                Self::Integer {
                    bits: Bits::Bit16,
                    min,
                    max,
                },
                Value::I16(val),
            ) if *val as i64 >= min.unwrap_or(0)
                && *val as i64 <= max.unwrap_or(i16::MAX as i64) =>
            {
                Ok(())
            }
            (
                Self::Integer {
                    bits: Bits::Bit8,
                    min,
                    max,
                },
                Value::I8(val),
            ) if *val as i64 >= min.unwrap_or(0)
                && *val as i64 <= max.unwrap_or(i8::MAX as i64) =>
            {
                Ok(())
            }
            (
                Self::Float {
                    bits: Bits::Bit64,
                    min,
                    max,
                },
                Value::F64(val),
            ) if *val >= min.unwrap_or(0.0) && *val <= max.unwrap_or(f64::MAX) => Ok(()),
            (
                Self::Float {
                    bits: Bits::Bit32,
                    min,
                    max,
                },
                Value::F32(val),
            ) if *val as f64 >= min.unwrap_or(0.0)
                && *val as f64 <= max.unwrap_or(f32::MAX as f64) =>
            {
                Ok(())
            }

            (Self::Enum { values }, Value::U8(val)) if values.contains(val) => Ok(()),
            (Self::String(max_len), Value::Str(string)) if string.len() <= *max_len as usize => {
                Ok(())
            }
            (Self::Bytes(max_len), Value::Bytes(bytes)) if bytes.len() <= *max_len as usize => {
                Ok(())
            }

            // TODO: other types when added to metadata::Value
            _ => Err(SchemaError::InvalidValue(value.clone())),
        }
    }
}

impl Field {
    pub fn validate(&self, field_type: Type, metadata: &Metadata) -> Result<(), SchemaError> {
        let count = metadata
            .iter()
            .filter_map(|m| {
                if m.id == field_type {
                    Some(&m.val)
                } else {
                    None
                }
            })
            .try_fold(0, |acc, val| {
                self.0.validate(&val).and_then(|_| Ok(acc + 1))
            })
            .map_err(|e| SchemaError::InvalidField(field_type, Box::new(e)))?;

        self.1.check_count(count).map_err(|e| {
            SchemaError::InvalidField(field_type, Box::new(SchemaError::OccurencesNotMet(e)))
        })
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::super::types::*;
    use super::{Field, FieldFormat};
    use crate::rgb::metadata::{self, Metadata, Type, Value};

    #[test]
    fn test_validate_unsigned_256() {
        let field_format = FieldFormat::Unsigned {
            bits: Bits::Bit256,
            min: None,
            max: None,
        };
        let value = Value::U256(Default::default());
        field_format.validate(&value).unwrap();
    }
    #[test]
    #[should_panic(expected = "MinMaxBoundsOnLargeInt")]
    fn test_validate_unsigned_256_bounds() {
        let field_format = FieldFormat::Unsigned {
            bits: Bits::Bit256,
            min: Some(0),
            max: None,
        };
        let value = Value::U256(Default::default());
        field_format.validate(&value).unwrap();
    }

    #[test]
    fn test_validate_unsigned_64() {
        let field_format = FieldFormat::Unsigned {
            bits: Bits::Bit64,
            min: None,
            max: None,
        };
        let value = Value::U64(42424242);
        field_format.validate(&value).unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidValue(U64(42))")]
    fn test_validate_unsigned_64_min() {
        let field_format = FieldFormat::Unsigned {
            bits: Bits::Bit64,
            min: Some(69),
            max: None,
        };
        let value = Value::U64(42);
        field_format.validate(&value).unwrap();
    }
    #[test]
    fn test_validate_unsigned_64_min_max() {
        let field_format = FieldFormat::Unsigned {
            bits: Bits::Bit64,
            min: Some(42),
            max: Some(69),
        };
        let value = Value::U64(50);
        field_format.validate(&value).unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidValue(U32(42424242))")]
    fn test_validate_unsigned_64_wrong_type() {
        let field_format = FieldFormat::Unsigned {
            bits: Bits::Bit64,
            min: None,
            max: None,
        };
        let value = Value::U32(42424242);
        field_format.validate(&value).unwrap();
    }

    #[test]
    fn test_validate_enum() {
        let field_format = FieldFormat::Enum {
            values: vec![0, 1, 2, 3],
        };
        let value = Value::U8(2);
        field_format.validate(&value).unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidValue(U8(42))")]
    fn test_validate_enum_missing() {
        let field_format = FieldFormat::Enum {
            values: vec![0, 1, 2, 3],
        };
        let value = Value::U8(42);
        field_format.validate(&value).unwrap();
    }

    #[test]
    fn test_validate_string() {
        let field_format = FieldFormat::String(5);
        let value = Value::Str("test".into());
        field_format.validate(&value).unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidValue(Str(\"testtest\"))")]
    fn test_validate_string_too_long() {
        let field_format = FieldFormat::String(5);
        let value = Value::Str("testtest".into());
        field_format.validate(&value).unwrap();
    }

    #[test]
    fn test_validate_bytes() {
        let field_format = FieldFormat::Bytes(5);
        let value = Value::Bytes(vec![0x00, 0x11].into_boxed_slice());
        field_format.validate(&value).unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidValue(Bytes([0, 0, 0, 0]))")]
    fn test_validate_bytes_too_long() {
        let field_format = FieldFormat::Bytes(3);
        let value = Value::Bytes(vec![0x00; 4].into_boxed_slice());
        field_format.validate(&value).unwrap();
    }

    #[test]
    fn test_validate_metadata_empty() {
        let field = Field(
            FieldFormat::Unsigned {
                bits: Bits::Bit64,
                min: None,
                max: None,
            },
            Occurences::NoneOrOnce,
        );
        let metadata = Metadata::from_inner(vec![]);
        field.validate(Type(0), &metadata).unwrap()
    }

    #[test]
    fn test_validate_metadata_simple() {
        let field = Field(
            FieldFormat::Unsigned {
                bits: Bits::Bit64,
                min: None,
                max: None,
            },
            Occurences::NoneOrOnce,
        );
        let metadata = Metadata::from_inner(vec![metadata::Field {
            id: Type(0),
            val: Value::U64(42),
        }]);
        field.validate(Type(0), &metadata).unwrap()
    }

    #[test]
    #[should_panic(
        expected = "InvalidField(Type(0), OccurencesNotMet(OccurencesError { expected: NoneOrOnce, found: 2 })"
    )]
    fn test_validate_metadata_fail_too_many() {
        let field = Field(
            FieldFormat::Unsigned {
                bits: Bits::Bit64,
                min: None,
                max: None,
            },
            Occurences::NoneOrOnce,
        );
        let metadata = Metadata::from_inner(vec![
            metadata::Field {
                id: Type(0),
                val: Value::U64(0),
            },
            metadata::Field {
                id: Type(0),
                val: Value::U64(42),
            },
        ]);
        field.validate(Type(0), &metadata).unwrap()
    }

    #[test]
    #[should_panic(expected = "InvalidField(Type(0), InvalidValue(U32(42)))")]
    fn test_validate_metadata_fail_invalid_value() {
        let field = Field(
            FieldFormat::Unsigned {
                bits: Bits::Bit64,
                min: None,
                max: None,
            },
            Occurences::NoneOrOnce,
        );
        let metadata = Metadata::from_inner(vec![metadata::Field {
            id: Type(0),
            val: Value::U32(42),
        }]);
        field.validate(Type(0), &metadata).unwrap()
    }
}