use std::collections::{HashMap, HashSet};
use std::ptr::{addr_of, null};
use inflector::Inflector;
use crate::action::action::ActionType;
use crate::core::connector::{ConnectorBuilder};
use crate::core::field::*;
use crate::core::builders::field_builder::FieldBuilder;
use crate::core::builders::action_builder::ActionBuilder;
use crate::core::builders::model_index_builder::{ModelIndexBuilder};
use crate::core::builders::permission_builder::PermissionBuilder;
use crate::core::field::ReadRule::NoRead;
use crate::core::field::Store::{Calculated, Temp};
use crate::core::field::WriteRule::NoWrite;
use crate::core::model::{ModelIndex, ModelIndexItem, Model, ModelIndexType};


pub struct ModelBuilder {
    pub(crate) name: &'static str,
    pub(crate) table_name: &'static str,
    pub(crate) url_segment_name: &'static str,
    pub(crate) localized_name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) identity: bool,
    pub(crate) field_builders: Vec<FieldBuilder>,
    pub(crate) actions: HashSet<ActionType>,
    pub(crate) permission: Option<PermissionBuilder>,
    pub(crate) primary: Option<ModelIndex>,
    pub(crate) indices: Vec<ModelIndex>,
    connector_builder: * const Box<dyn ConnectorBuilder>,
}

impl ModelBuilder {

    pub(crate) fn new(name: &'static str, connector_builder: &Box<dyn ConnectorBuilder>) -> Self {
        Self {
            name,
            table_name: "",
            url_segment_name: "",
            localized_name: "",
            description: "",
            identity: false,
            field_builders: Vec::new(),
            actions: ActionType::default(),
            permission: None,
            primary: None,
            indices: Vec::new(),
            connector_builder
        }
    }

    fn connector_builder(&self) -> &Box<dyn ConnectorBuilder> {
        unsafe {
            &*self.connector_builder
        }
    }

    pub fn table_name(&mut self, table_name: &'static str) -> &mut Self {
        self.table_name = table_name;
        self
    }

    pub fn url_segment_name(&mut self, url_segment_name: &'static str) -> &mut Self {
        self.url_segment_name = url_segment_name;
        self
    }

    pub fn localized_name(&mut self, localized_name: &'static str) -> &mut Self {
        self.localized_name = localized_name;
        self
    }

    pub fn description(&mut self, description: &'static str) -> &mut Self {
        self.description = description;
        self
    }

    pub fn identity(&mut self) -> &mut Self {
        self.identity = true;
        self.actions.insert(ActionType::SignIn);
        self
    }

    pub fn field<F: Fn(&mut FieldBuilder)>(&mut self, name: &'static str, build: F) -> &mut Self {
        let mut f = FieldBuilder::new(name, self.connector_builder());
        build(&mut f);
        self.field_builders.push(f);
        self
    }

    pub fn internal(&mut self) -> &mut Self {
        self.actions = HashSet::new();
        self
    }

    pub fn enable<F: Fn(&mut ActionBuilder)>(&mut self, build: F) -> &mut Self {
        self.internal();
        let mut action_builder = ActionBuilder::new();
        build(&mut action_builder);
        self.actions = action_builder.actions.clone();
        if self.identity {
            self.actions.insert(ActionType::SignIn);
        }
        self
    }

    pub fn disable<F: Fn(&mut ActionBuilder)>(&mut self, build: F) -> &mut Self {
        let mut action_builder = ActionBuilder::new();
        build(&mut action_builder);
        self.actions = HashSet::from_iter(self.actions.difference(&action_builder.actions).map(|x| *x));
        if self.identity {
            self.actions.insert(ActionType::SignIn);
        }
        self
    }

    pub fn permissions<F: Fn(&mut PermissionBuilder)>(&mut self, build: F) -> &mut Self {
        let mut permission_builder = PermissionBuilder::new();
        build(&mut permission_builder);
        self.permission = Some(permission_builder);
        self
    }

    pub fn primary<I, T>(&mut self, keys: I) -> &mut Self where I: IntoIterator<Item = T>, T: Into<String> {
        let string_keys: Vec<String> = keys.into_iter().map(Into::into).collect();
        let name = string_keys.join("_");
        let items = string_keys.iter().map(|k| {
            ModelIndexItem { field_name: k.to_string(), sort: Sort::Asc, len: None }
        }).collect();
        let index = ModelIndex {
            index_type: ModelIndexType::Primary,
            name,
            items
        };
        self.primary = Some(index);
        self
    }

    pub fn primary_settings<F: Fn(&mut ModelIndexBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = ModelIndexBuilder::new(ModelIndexType::Primary);
        build(&mut builder);
        self.primary = Some(builder.build());
        self
    }

    pub fn index<I, T>(&mut self, keys: I) -> &mut Self where I: IntoIterator<Item = T>, T: Into<String> {
        let string_keys: Vec<String> = keys.into_iter().map(Into::into).collect();
        let name = string_keys.join("_");
        let items = string_keys.iter().map(|k| {
            ModelIndexItem { field_name: k.to_string(), sort: Sort::Asc, len: None }
        }).collect();
        let index = ModelIndex {
            index_type: ModelIndexType::Index,
            name,
            items
        };
        self.indices.push(index);
        self
    }

    pub fn index_settings<F: Fn(&mut ModelIndexBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = ModelIndexBuilder::new(ModelIndexType::Index);
        build(&mut builder);
        self.indices.push(builder.build());
        self
    }

    pub fn unique<I, T>(&mut self, keys: I) -> &mut Self where I: IntoIterator<Item = T>, T: Into<String> {
        let string_keys: Vec<String> = keys.into_iter().map(Into::into).collect();
        let name = string_keys.join("_");
        let items = string_keys.iter().map(|k| {
            ModelIndexItem { field_name: k.to_string(), sort: Sort::Asc, len: None }
        }).collect();
        let index = ModelIndex {
            index_type: ModelIndexType::Unique,
            name,
            items
        };
        self.indices.push(index);
        self
    }

    pub fn unique_settings<F: Fn(&mut ModelIndexBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = ModelIndexBuilder::new(ModelIndexType::Unique);
        build(&mut builder);
        self.indices.push(builder.build());
        self
    }

    pub(crate) fn build(&self, connector_builder: &Box<dyn ConnectorBuilder>) -> Model {
        let input_keys = Self::allowed_input_keys(self);
        let save_keys = Self::allowed_save_keys(self);
        let output_keys = Self::allowed_output_keys(self);
        let get_value_keys = Self::get_get_value_keys(self);
        let query_keys = Self::get_query_keys(self);
        let unique_query_keys = Self::get_unique_query_keys(self);
        let auth_identity_keys = Self::get_auth_identity_keys(self);
        let auth_by_keys = Self::get_auth_by_keys(self);
        let fields_vec: Vec<Field> = self.field_builders.iter().map(|fb| { fb.build(connector_builder) }).collect();
        let mut fields_map: HashMap<String, * const Field> = HashMap::new();
        let mut primary_field: * const Field = null();
        let mut index_fields: Vec<* const Field> = Vec::new();
        let mut primary = self.primary.clone();
        let mut indices = self.indices.clone();
        for field in fields_vec.iter() {
            let addr = addr_of!(*field);
            fields_map.insert(field.name.clone(), addr);
            if field.primary {
                primary_field = addr_of!(*field);
                primary = Some(ModelIndex {
                    index_type: ModelIndexType::Primary,
                    name: "".to_string(),
                    items: vec![
                        ModelIndexItem {
                            field_name: field.column_name().clone(),
                            sort: Sort::Asc,
                            len: None
                        }
                    ]
                });
            }
            if field.index != FieldIndex::NoIndex {
                index_fields.push(addr);
                match &field.index {
                    FieldIndex::Index(settings) => {
                        indices.push(ModelIndex {
                            index_type: ModelIndexType::Index,
                            name: if settings.name.is_some() { settings.name.as_ref().unwrap().clone() } else { field.column_name().clone() },
                            items: vec![
                                ModelIndexItem {
                                    field_name: field.column_name().clone(),
                                    sort: settings.sort,
                                    len: settings.length
                                }
                            ]
                        })
                    }
                    FieldIndex::Unique(settings) => {
                        indices.push(ModelIndex {
                            index_type: ModelIndexType::Unique,
                            name: if settings.name.is_some() { settings.name.as_ref().unwrap().clone() } else { field.column_name().clone() },
                            items: vec![
                                ModelIndexItem {
                                    field_name: field.column_name().clone(),
                                    sort: settings.sort,
                                    len: settings.length
                                }
                            ]
                        })
                    }
                    _ => { }
                }
            }
        }

        if primary.is_none() {
            panic!("Model '{}' must has a primary field.", self.name);
        }

        Model {
            name: self.name,
            table_name: if self.table_name == "" { self.name.to_lowercase().to_plural() } else { self.table_name.to_string() },
            url_segment_name: if self.url_segment_name == "" { self.name.to_kebab_case().to_plural() } else { self.url_segment_name.to_string() },
            localized_name: self.localized_name,
            description: self.description,
            identity: self.identity,
            actions: self.actions.clone(),
            permission: if let Some(builder) = &self.permission { Some(builder.build()) } else { None },
            fields_vec,
            fields_map,
            primary: primary.unwrap(),
            indices: indices.clone(),
            primary_field,
            index_fields,
            input_keys,
            save_keys,
            output_keys,
            get_value_keys,
            query_keys,
            unique_query_keys,
            auth_identity_keys,
            auth_by_keys
        }
    }

    fn allowed_input_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.write_rule != NoWrite })
            .map(|f| { f.name.clone() })
            .collect()
    }

    fn allowed_save_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.store != Calculated && f.store != Temp })
            .map(|f| { f.name.clone() })
            .collect()
    }

    fn allowed_output_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.read_rule != NoRead })
            .map(|f| { f.name.clone() })
            .collect()
    }

    pub(crate) fn get_get_value_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .map(|f| { f.name.clone() })
            .collect()
    }

    pub(crate) fn get_query_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.query_ability == QueryAbility::Queryable })
            .map(|f| { f.name.clone() })
            .collect()
    }

    pub(crate) fn get_unique_query_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            //.filter(|&f| { f.query_ability == QueryAbility::Queryable && (f.index == FieldIndex::Unique || f.primary == true) })
            .map(|f| { f.name.clone() })
            .collect()
    }

    pub(crate) fn get_auth_identity_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.auth_identity == true })
            .map(|f| { f.name.clone() })
            .collect()
    }

    pub(crate) fn get_auth_by_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.auth_by == true })
            .map(|f| { f.name.clone() })
            .collect()
    }
}
