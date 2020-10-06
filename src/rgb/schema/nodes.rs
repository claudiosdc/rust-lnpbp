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

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use super::{
    ExtensionAbi, ExtensionAction, FieldType, GenesisAbi, GenesisAction,
    NodeAction, Occurences, Procedure, TransitionAbi, TransitionAction,
};

// Here we can use usize since encoding/decoding makes sure that it's u16
pub type AssignmentsType = usize;
pub type ValenciesType = usize;
pub type MetadataStructure = BTreeMap<FieldType, Occurences<u16>>;
pub type ValenciesStructure = BTreeSet<ValenciesType>;
pub type SealsStructure = BTreeMap<AssignmentsType, Occurences<u16>>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
/// Node type: genesis, extensions and state transitions
pub enum NodeType {
    /// Genesis node: single node per contract, defining contract and
    /// committing to a specific schema and underlying chain hash
    #[display("genesis")]
    Genesis,

    /// Multiple points for decentralized & unowned contract extension,
    /// committing either to a genesis or some state transition via their
    /// valencies
    #[display("extension")]
    Extension,

    /// State transition performing owned change to the state data and
    /// committing to (potentially multiple) ancestors (i.e. genesis,
    /// extensions and/or  other state transitions) via spending
    /// corresponding transaction outputs assigned some state by ancestors
    #[display("transition")]
    StateTransition,
}

/// Trait defining common API for all node type schemata
pub trait NodeSchema {
    type Action: NodeAction;

    fn node_type(&self) -> NodeType;
    fn metadata(&self) -> &MetadataStructure;
    fn closes(&self) -> &SealsStructure;
    fn extends(&self) -> &ValenciesStructure;
    fn defines(&self) -> &SealsStructure;
    fn valencies(&self) -> &ValenciesStructure;
    fn abi(&self) -> &BTreeMap<Self::Action, Procedure>;
}

#[derive(Clone, PartialEq, Debug, Display, AsAny)]
#[display(Debug)]
pub struct GenesisSchema {
    pub metadata: MetadataStructure,
    pub defines: SealsStructure,
    pub valencies: ValenciesStructure,
    pub abi: GenesisAbi,
}

#[derive(Clone, PartialEq, Debug, Display, AsAny)]
#[display(Debug)]
pub struct ExtensionSchema {
    pub metadata: MetadataStructure,
    pub extends: ValenciesStructure,
    pub defines: SealsStructure,
    pub valencies: ValenciesStructure,
    pub abi: ExtensionAbi,
}

#[derive(Clone, PartialEq, Debug, Display, AsAny)]
#[display(Debug)]
pub struct TransitionSchema {
    pub metadata: MetadataStructure,
    pub closes: SealsStructure,
    pub defines: SealsStructure,
    pub valencies: ValenciesStructure,
    pub abi: TransitionAbi,
}

lazy_static! {
    static ref EMPTY_SEALS: SealsStructure = SealsStructure::new();
    static ref EMPTY_VALENCIES: ValenciesStructure = ValenciesStructure::new();
}

impl NodeSchema for GenesisSchema {
    type Action = GenesisAction;

    #[inline]
    fn node_type(&self) -> NodeType {
        NodeType::Genesis
    }
    #[inline]
    fn metadata(&self) -> &MetadataStructure {
        &self.metadata
    }
    #[inline]
    fn closes(&self) -> &SealsStructure {
        &EMPTY_SEALS
    }
    #[inline]
    fn extends(&self) -> &ValenciesStructure {
        &EMPTY_VALENCIES
    }
    #[inline]
    fn defines(&self) -> &SealsStructure {
        &self.defines
    }
    #[inline]
    fn valencies(&self) -> &ValenciesStructure {
        &self.valencies
    }
    #[inline]
    fn abi(&self) -> &BTreeMap<Self::Action, Procedure> {
        &self.abi
    }
}

impl NodeSchema for ExtensionSchema {
    type Action = ExtensionAction;

    #[inline]
    fn node_type(&self) -> NodeType {
        NodeType::Extension
    }
    #[inline]
    fn metadata(&self) -> &MetadataStructure {
        &self.metadata
    }
    #[inline]
    fn closes(&self) -> &SealsStructure {
        &EMPTY_SEALS
    }
    #[inline]
    fn extends(&self) -> &ValenciesStructure {
        &self.extends
    }
    #[inline]
    fn defines(&self) -> &SealsStructure {
        &self.defines
    }
    #[inline]
    fn valencies(&self) -> &ValenciesStructure {
        &self.valencies
    }
    #[inline]
    fn abi(&self) -> &BTreeMap<Self::Action, Procedure> {
        &self.abi
    }
}

impl NodeSchema for TransitionSchema {
    type Action = TransitionAction;

    #[inline]
    fn node_type(&self) -> NodeType {
        NodeType::StateTransition
    }
    #[inline]
    fn metadata(&self) -> &MetadataStructure {
        &self.metadata
    }
    #[inline]
    fn closes(&self) -> &SealsStructure {
        &self.closes
    }
    #[inline]
    fn extends(&self) -> &ValenciesStructure {
        &EMPTY_VALENCIES
    }
    #[inline]
    fn defines(&self) -> &SealsStructure {
        &self.defines
    }
    #[inline]
    fn valencies(&self) -> &ValenciesStructure {
        &self.valencies
    }
    #[inline]
    fn abi(&self) -> &BTreeMap<Self::Action, Procedure> {
        &self.abi
    }
}

mod strict_encoding {
    use super::*;
    use crate::strict_encoding::{Error, StrictDecode, StrictEncode};

    impl StrictEncode for GenesisSchema {
        type Error = Error;

        fn strict_encode<E: io::Write>(
            &self,
            mut e: E,
        ) -> Result<usize, Error> {
            let mut len = 0usize;
            len += self.metadata.strict_encode(&mut e)?;
            len += self.defines.strict_encode(&mut e)?;
            len += self.valencies.strict_encode(&mut e)?;
            len += self.abi.strict_encode(&mut e)?;
            // We keep this parameter for future script extended info (like ABI)
            len += Vec::<u8>::new().strict_encode(&mut e)?;
            Ok(len)
        }
    }

    impl StrictDecode for GenesisSchema {
        type Error = Error;

        fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Error> {
            let me = Self {
                metadata: MetadataStructure::strict_decode(&mut d)?,
                defines: SealsStructure::strict_decode(&mut d)?,
                valencies: ValenciesStructure::strict_decode(&mut d)?,
                abi: GenesisAbi::strict_decode(&mut d)?,
            };
            // We keep this parameter for future script extended info (like ABI)
            let script = Vec::<u8>::strict_decode(&mut d)?;
            if !script.is_empty() {
                Err(Error::UnsupportedDataStructure(
                    "Scripting information is not yet supported",
                ))
            } else {
                Ok(me)
            }
        }
    }

    impl StrictEncode for ExtensionSchema {
        type Error = Error;

        fn strict_encode<E: io::Write>(
            &self,
            mut e: E,
        ) -> Result<usize, Error> {
            let mut len = 0usize;
            len += self.metadata.strict_encode(&mut e)?;
            len += self.extends.strict_encode(&mut e)?;
            len += self.defines.strict_encode(&mut e)?;
            len += self.valencies.strict_encode(&mut e)?;
            len += self.abi.strict_encode(&mut e)?;
            // We keep this parameter for future script extended info (like ABI)
            len += Vec::<u8>::new().strict_encode(&mut e)?;
            Ok(len)
        }
    }

    impl StrictDecode for ExtensionSchema {
        type Error = Error;

        fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Error> {
            let me = Self {
                metadata: MetadataStructure::strict_decode(&mut d)?,
                extends: ValenciesStructure::strict_decode(&mut d)?,
                defines: SealsStructure::strict_decode(&mut d)?,
                valencies: ValenciesStructure::strict_decode(&mut d)?,
                abi: ExtensionAbi::strict_decode(&mut d)?,
            };
            // We keep this parameter for future script extended info (like ABI)
            let script = Vec::<u8>::strict_decode(&mut d)?;
            if !script.is_empty() {
                Err(Error::UnsupportedDataStructure(
                    "Scripting information is not yet supported",
                ))
            } else {
                Ok(me)
            }
        }
    }

    impl StrictEncode for TransitionSchema {
        type Error = Error;

        fn strict_encode<E: io::Write>(
            &self,
            mut e: E,
        ) -> Result<usize, Error> {
            let mut len = 0usize;
            len += self.metadata.strict_encode(&mut e)?;
            len += self.closes.strict_encode(&mut e)?;
            len += self.defines.strict_encode(&mut e)?;
            len += self.valencies.strict_encode(&mut e)?;
            len += self.abi.strict_encode(&mut e)?;
            // We keep this parameter for future script extended info (like ABI)
            len += Vec::<u8>::new().strict_encode(&mut e)?;
            Ok(len)
        }
    }

    impl StrictDecode for TransitionSchema {
        type Error = Error;

        fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Error> {
            let me = Self {
                metadata: MetadataStructure::strict_decode(&mut d)?,
                closes: SealsStructure::strict_decode(&mut d)?,
                defines: SealsStructure::strict_decode(&mut d)?,
                valencies: ValenciesStructure::strict_decode(&mut d)?,
                abi: TransitionAbi::strict_decode(&mut d)?,
            };
            // We keep this parameter for future script extended info (like ABI)
            let script = Vec::<u8>::strict_decode(&mut d)?;
            if !script.is_empty() {
                Err(Error::UnsupportedDataStructure(
                    "Scripting information is not yet supported",
                ))
            } else {
                Ok(me)
            }
        }
    }
}

mod _verify {
    use super::*;
    use crate::rgb::schema::SchemaVerify;
    use crate::rgb::validation;
    use num_traits::ToPrimitive;

    impl<T> SchemaVerify for T
    where
        T: NodeSchema,
    {
        fn schema_verify(&self, root: &Self) -> validation::Status {
            let mut status = validation::Status::new();
            let node_type = self.node_type();

            for (field_type, occ) in self.metadata() {
                match root.metadata().get(field_type) {
                    None => status.add_failure(
                        validation::Failure::SchemaRootNoMetadataMatch(
                            node_type,
                            *field_type,
                        ),
                    ),
                    Some(root_occ) if occ != root_occ => status.add_failure(
                        validation::Failure::SchemaRootNoMetadataMatch(
                            node_type,
                            *field_type,
                        ),
                    ),
                    _ => &status,
                };
            }

            for (assignments_type, occ) in self.closes() {
                match root.closes().get(assignments_type) {
                    None => status.add_failure(
                        validation::Failure::SchemaRootNoClosedAssignmentsMatch(
                            node_type,
                            *assignments_type,
                        ),
                    ),
                    Some(root_occ) if occ != root_occ => status.add_failure(
                        validation::Failure::SchemaRootNoClosedAssignmentsMatch(
                            node_type,
                            *assignments_type,
                        ),
                    ),
                    _ => &status,
                };
            }

            for (assignments_type, occ) in self.defines() {
                match root.defines().get(assignments_type) {
                    None => status.add_failure(
                        validation::Failure::SchemaRootNoDefinedAssignmentsMatch(
                            node_type,
                            *assignments_type,
                        ),
                    ),
                    Some(root_occ) if occ != root_occ => status.add_failure(
                        validation::Failure::SchemaRootNoDefinedAssignmentsMatch(
                            node_type,
                            *assignments_type,
                        ),
                    ),
                    _ => &status,
                };
            }

            for valencies_type in self.extends() {
                if !root.extends().contains(valencies_type) {
                    status.add_failure(
                        validation::Failure::SchemaRootNoExtendedValenciesMatch(
                            node_type,
                            *valencies_type,
                        ),
                    );
                }
            }

            for valencies_type in self.valencies() {
                if !root.valencies().contains(valencies_type) {
                    status.add_failure(
                        validation::Failure::SchemaRootNoDefinedValenciesMatch(
                            node_type,
                            *valencies_type,
                        ),
                    );
                }
            }

            for (action, proc) in self.abi() {
                match root.abi().get(action) {
                    None => status.add_failure(
                        validation::Failure::SchemaRootNoAbiMatch {
                            node_type,
                            action_id: action.to_u16().expect(
                                "Action type can't exceed 16-bit integer",
                            ),
                        },
                    ),
                    Some(root_proc) if root_proc != proc => status.add_failure(
                        validation::Failure::SchemaRootNoAbiMatch {
                            node_type,
                            action_id: action.to_u16().expect(
                                "Action type can't exceed 16-bit integer",
                            ),
                        },
                    ),
                    _ => &status,
                };
            }

            status
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rgb::schema::script::StandardProcedure;
    use crate::rgb::schema::SchemaVerify;
    use crate::rgb::validation::Failure;
    use crate::strict_encoding::{test::*, StrictDecode};

    static GENESIS_SCHEMA: [u8; 109] = [
        0x4, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x0, 0xff, 0xd,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0xfe, 0x11, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x2, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x3, 0x0, 0xff, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0,
        0xfe, 0xc, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0x1, 0x0, 0x2,
        0x0, 0x3, 0x0, 0x4, 0x0, 0x1, 0x0, 0x0, 0xff, 0x1, 0x0, 0x0,
    ];

    static TRANSITION_SCHEMA: [u8; 155] = [
        0x4, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x0, 0xff, 0xd,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0xfe, 0x11, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x2, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x3, 0x0, 0xff, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0,
        0xfe, 0xc, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0x1, 0x0, 0x1,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x0, 0xff, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x4, 0x0, 0xfe, 0xc, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4,
        0x0, 0x1, 0x0, 0x2, 0x0, 0x3, 0x0, 0x4, 0x0, 0x1, 0x0, 0x0, 0xff, 0x1,
        0x0, 0x0,
    ];

    static EXTENSION_SCHEMA: [u8; 119] = [
        0x4, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x0, 0xff, 0xd,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0xfe, 0x11, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0x1, 0x0, 0x2, 0x0, 0x3, 0x0, 0x4, 0x0,
        0x4, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x0, 0xff, 0x19,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0xfe, 0xc, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x4, 0x0, 0x1, 0x0, 0x2, 0x0, 0x3, 0x0, 0x4, 0x0,
        0x1, 0x0, 0x0, 0xff, 0x1, 0x0, 0x0,
    ];

    #[test]
    fn test_nodeschema_encoding() {
        test_encode!(
            (GENESIS_SCHEMA, GenesisSchema),
            (TRANSITION_SCHEMA, TransitionSchema),
            (EXTENSION_SCHEMA, ExtensionSchema)
        );
    }

    #[test]
    fn test_node_for_genesis() {
        let genesis_schema =
            GenesisSchema::strict_decode(&GENESIS_SCHEMA[..]).unwrap();

        let mut valencies = ValenciesStructure::new();
        valencies.insert(1usize);
        valencies.insert(2usize);
        valencies.insert(3usize);
        valencies.insert(4usize);

        let mut genesis_abi = GenesisAbi::new();
        genesis_abi.insert(
            GenesisAction::NoOp,
            Procedure::Standard(StandardProcedure::ConfidentialAmount),
        );

        assert_eq!(genesis_schema.node_type(), NodeType::Genesis);
        assert_eq!(
            genesis_schema.metadata().get(&2usize).unwrap(),
            &Occurences::NoneOrOnce
        );
        assert_eq!(genesis_schema.closes(), &SealsStructure::new());
        assert_eq!(genesis_schema.extends(), &ValenciesStructure::new());
        assert_eq!(
            genesis_schema.defines().get(&3usize).unwrap(),
            &Occurences::OnceOrUpTo(Some(25u16))
        );
        assert_eq!(genesis_schema.valencies(), &valencies);
        assert_eq!(genesis_schema.abi(), &genesis_abi);
    }

    #[test]
    fn test_node_for_transition() {
        let transition_schema =
            TransitionSchema::strict_decode(&TRANSITION_SCHEMA[..]).unwrap();

        let mut valencies = ValenciesStructure::new();
        valencies.insert(1usize);
        valencies.insert(2usize);
        valencies.insert(3usize);
        valencies.insert(4usize);

        let mut transition_abi = TransitionAbi::new();
        transition_abi.insert(
            TransitionAction::GenerateBlank,
            Procedure::Standard(StandardProcedure::ConfidentialAmount),
        );

        assert_eq!(transition_schema.node_type(), NodeType::StateTransition);
        assert_eq!(
            transition_schema.metadata().get(&2usize).unwrap(),
            &Occurences::NoneOrOnce
        );
        assert_eq!(
            transition_schema.closes().get(&3usize).unwrap(),
            &Occurences::OnceOrUpTo(Some(25u16))
        );
        assert_eq!(transition_schema.extends(), &ValenciesStructure::new());
        assert_eq!(
            transition_schema.defines().get(&3usize).unwrap(),
            &Occurences::OnceOrUpTo(Some(25u16))
        );
        assert_eq!(transition_schema.valencies(), &valencies);
        assert_eq!(transition_schema.abi(), &transition_abi);
    }

    #[test]
    fn test_node_for_extension() {
        let extension_schema =
            ExtensionSchema::strict_decode(&EXTENSION_SCHEMA[..]).unwrap();

        let mut valencies = ValenciesStructure::new();
        valencies.insert(1usize);
        valencies.insert(2usize);
        valencies.insert(3usize);
        valencies.insert(4usize);

        let mut extension_abi = ExtensionAbi::new();
        extension_abi.insert(
            ExtensionAction::NoOp,
            Procedure::Standard(StandardProcedure::ConfidentialAmount),
        );

        assert_eq!(extension_schema.node_type(), NodeType::Extension);
        assert_eq!(
            extension_schema.metadata().get(&2usize).unwrap(),
            &Occurences::NoneOrOnce
        );
        assert_eq!(extension_schema.closes(), &SealsStructure::new());
        assert_eq!(extension_schema.extends(), &valencies);
        assert_eq!(
            extension_schema.defines().get(&3usize).unwrap(),
            &Occurences::OnceOrUpTo(Some(25u16))
        );
        assert_eq!(extension_schema.valencies(), &valencies);
        assert_eq!(extension_schema.abi(), &extension_abi);
    }

    #[test]
    fn test_validation() {
        // Create Two Metadata Structures
        let mut metadata_structures = MetadataStructure::new();
        metadata_structures.insert(1 as FieldType, Occurences::Once);
        metadata_structures.insert(2 as FieldType, Occurences::NoneOrOnce);
        metadata_structures
            .insert(3 as FieldType, Occurences::OnceOrUpTo(Some(13u16)));
        metadata_structures
            .insert(4 as FieldType, Occurences::NoneOrUpTo(Some(17u16)));

        let mut metadata_structures2 = MetadataStructure::new();
        metadata_structures2.insert(1 as FieldType, Occurences::Once);
        metadata_structures2.insert(2 as FieldType, Occurences::NoneOrOnce);
        metadata_structures2
            .insert(3 as FieldType, Occurences::OnceOrUpTo(None));
        metadata_structures2
            .insert(4 as FieldType, Occurences::NoneOrUpTo(Some(15u16)));

        // Create Two Seal Structures
        let mut seal_structures = SealsStructure::new();
        seal_structures.insert(1 as AssignmentsType, Occurences::Once);
        seal_structures.insert(2 as AssignmentsType, Occurences::NoneOrOnce);
        seal_structures
            .insert(3 as AssignmentsType, Occurences::OnceOrUpTo(Some(25u16)));
        seal_structures
            .insert(4 as AssignmentsType, Occurences::NoneOrUpTo(Some(12u16)));

        let mut seal_structures2 = SealsStructure::new();
        seal_structures2.insert(1 as AssignmentsType, Occurences::Once);
        seal_structures2.insert(2 as AssignmentsType, Occurences::NoneOrOnce);
        seal_structures2
            .insert(3 as AssignmentsType, Occurences::OnceOrUpTo(None));
        seal_structures2
            .insert(4 as AssignmentsType, Occurences::NoneOrUpTo(Some(30u16)));

        // Create Two Valency structure
        let mut valency_structure = ValenciesStructure::new();
        valency_structure.insert(1 as ValenciesType);
        valency_structure.insert(2 as ValenciesType);
        valency_structure.insert(3 as ValenciesType);
        valency_structure.insert(4 as ValenciesType);

        let mut valency_structure2 = ValenciesStructure::new();
        valency_structure2.insert(1 as ValenciesType);
        valency_structure2.insert(5 as ValenciesType);
        valency_structure2.insert(3 as ValenciesType);
        valency_structure2.insert(4 as ValenciesType);

        // Create the required ABIs
        let mut transition_abi = TransitionAbi::new();
        transition_abi.insert(
            TransitionAction::GenerateBlank,
            Procedure::Standard(StandardProcedure::ConfidentialAmount),
        );

        let mut transition_abi2 = TransitionAbi::new();
        transition_abi2.insert(
            TransitionAction::GenerateBlank,
            Procedure::Standard(StandardProcedure::Prunning),
        );

        let mut extension_abi = ExtensionAbi::new();
        extension_abi.insert(
            ExtensionAction::NoOp,
            Procedure::Standard(StandardProcedure::ConfidentialAmount),
        );

        // Create Four Unequal Transition and Extension Structures
        let transtion_schema = TransitionSchema {
            metadata: metadata_structures.clone(),
            closes: seal_structures.clone(),
            defines: seal_structures.clone(),
            valencies: valency_structure.clone(),
            abi: transition_abi.clone(),
        };

        let transtion_schema2 = TransitionSchema {
            metadata: metadata_structures2.clone(),
            closes: seal_structures2.clone(),
            defines: seal_structures2.clone(),
            valencies: valency_structure2.clone(),
            abi: transition_abi2.clone(),
        };

        let extension_schema = ExtensionSchema {
            metadata: metadata_structures.clone(),
            extends: valency_structure.clone(),
            defines: seal_structures.clone(),
            valencies: valency_structure.clone(),
            abi: extension_abi.clone(),
        };

        let extension_schema2 = ExtensionSchema {
            metadata: metadata_structures.clone(),
            extends: valency_structure2.clone(),
            defines: seal_structures.clone(),
            valencies: valency_structure2.clone(),
            abi: extension_abi.clone(),
        };

        // Create the expected failure results
        let transition_failures = vec![
            Failure::SchemaRootNoMetadataMatch(NodeType::StateTransition, 3),
            Failure::SchemaRootNoMetadataMatch(NodeType::StateTransition, 4),
            Failure::SchemaRootNoClosedAssignmentsMatch(
                NodeType::StateTransition,
                3,
            ),
            Failure::SchemaRootNoClosedAssignmentsMatch(
                NodeType::StateTransition,
                4,
            ),
            Failure::SchemaRootNoDefinedAssignmentsMatch(
                NodeType::StateTransition,
                3,
            ),
            Failure::SchemaRootNoDefinedAssignmentsMatch(
                NodeType::StateTransition,
                4,
            ),
            Failure::SchemaRootNoDefinedValenciesMatch(
                NodeType::StateTransition,
                2,
            ),
            Failure::SchemaRootNoAbiMatch {
                node_type: NodeType::StateTransition,
                action_id: 0,
            },
        ];

        let extension_failures = vec![
            Failure::SchemaRootNoExtendedValenciesMatch(NodeType::Extension, 2),
            Failure::SchemaRootNoDefinedValenciesMatch(NodeType::Extension, 2),
        ];

        // Assert failures matches with expectation
        assert_eq!(
            transtion_schema.schema_verify(&transtion_schema2).failures,
            transition_failures
        );
        assert_eq!(
            extension_schema.schema_verify(&extension_schema2).failures,
            extension_failures
        );
    }

    #[test]
    #[should_panic(expected = "UnsupportedDataStructure")]
    fn test_error_genesis() {
        let mut genesis_byte = GENESIS_SCHEMA.clone().to_vec();
        genesis_byte[107] = 2u8;
        genesis_byte[108] = 0u8;
        genesis_byte.push(1u8);
        genesis_byte.push(2u8);

        GenesisSchema::strict_decode(&genesis_byte[..]).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnsupportedDataStructure")]
    fn test_error_transition() {
        let mut transition_bytes = TRANSITION_SCHEMA.clone().to_vec();
        transition_bytes[153] = 2u8;
        transition_bytes[154] = 0u8;
        transition_bytes.push(1u8);
        transition_bytes.push(2u8);

        TransitionSchema::strict_decode(&transition_bytes[..]).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnsupportedDataStructure")]
    fn test_error_extension() {
        let mut extension_bytes = EXTENSION_SCHEMA.clone().to_vec();
        extension_bytes[117] = 2u8;
        extension_bytes[118] = 0u8;
        extension_bytes.push(1u8);
        extension_bytes.push(2u8);

        ExtensionSchema::strict_decode(&extension_bytes[..]).unwrap();
    }
}
