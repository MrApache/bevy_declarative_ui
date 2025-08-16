mod component;
mod resource;

use crate::codegen::Module;
use crate::r#static::required::RequiredBinding;
use bevy_declarative_ui_parser::utils::GetOrInsertEmpty;
use bevy_declarative_ui_parser::values::bindings::filter::{Filter, Filters};
use bevy_declarative_ui_parser::values::bindings::params::BaseParams;
use bevy_declarative_ui_parser::values::bindings::{BindingKind, BindingMode};
use std::collections::HashMap;

fn create_observers(bindings: &HashMap<String, Vec<RequiredBinding>>) -> ObserverCollection {
    let mut collection = ObserverCollection::default();
    bindings.iter().for_each(|(id, bindings)| {
        bindings.iter().for_each(|binding| {
            //match &binding.inner {
            //    BindingKind::Component => {
            //        let observer = create_observer(&binding, &binding..base, id.clone());
            //        let (list, _) = collection.components.get_or_insert(&params.base.target, ||init_cmp_hash_map(&params.filters)).get_mut(&params.base.mode).unwrap();
            //        list.push(observer);
            //    },
            //    BindingKind::Resource => {
            //        let observer = create_observer(&binding, &params, id.clone());
            //        collection.resources.get_or_insert(&params.target, init_res_hash_map).get_mut(&params.mode).unwrap().push(observer);
            //    }
            //}
        });
    });

    collection
}

fn create_observer<'a>(
    binding: &'a RequiredBinding,
    params: &BaseParams,
    id: String,
) -> Observer<'a> {
    /*
    let (get, set) = match params.mode {
        BindingMode::Read => {
            let mut get = params.path.to_string();
            let set = binding.field_name.clone();

            if let Some(converter) = &params.converter {
                get.push_str(&format!(".convert_to({converter}::default())"))
            }

            (get, set)
        },
        BindingMode::ReadWrite => {
            let mut get = params.path.to_string();
            let mut set = binding.field_name.clone();

            if let Some(converter) = &params.converter {
                //get.push_str(&format!(".convert_to({}::default())", params.converter()));
                get.push_str(&format!(".convert_from({converter}::default())"));
            }

            (get, set)
        },
        BindingMode::Write => {
            let mut get = binding.field_name.clone();
            let set = params.path.to_string();

            if let Some(converter) = &params.converter {
                get.push_str(&format!(".convert_from({converter}::default())"));
            }

            (get, set)
        },
        BindingMode::ReadOnce => unreachable!(),
    };

    Observer {
        target: &binding.component,
        set,
        get,
        id,
    }
    */

    Observer {
        target: &binding.component,
        id,
        set: String::new(),
        get: String::new(),
    }
}

fn init_res_hash_map<'a>() -> HashMap<BindingMode, Vec<Observer<'a>>> {
    let mut map = HashMap::new();
    map.insert(BindingMode::Read, vec![]);
    map.insert(BindingMode::Write, vec![]);
    map.insert(BindingMode::ReadWrite, vec![]);
    map
}

fn init_cmp_hash_map<'a>(filters: &Filters) -> HashMap<BindingMode, (Vec<Observer<'a>>, Filters)> {
    let mut map = HashMap::new();
    map.insert(BindingMode::Read, (vec![], filters.clone()));
    map.insert(BindingMode::Write, (vec![], filters.clone()));
    map.insert(BindingMode::ReadWrite, (vec![], filters.clone()));
    map
}

pub(super) fn binding_printer(bindings: &HashMap<String, Vec<RequiredBinding>>) -> Module {
    let mut module = Module::new("bindings");
    module.with_using("bevy::prelude::*");
    module.with_using("super::ids::*");

    let collection = create_observers(bindings);
    collection.resources.iter().for_each(|(resource, maps)| {
        maps.iter().for_each(|map| {
            if map.1.is_empty() {
                return;
            }
            let function = match map.0 {
                BindingMode::Read => resource::binding_resource_read_printer(resource, map.1),
                BindingMode::Write => resource::binding_resource_write_printer(resource, map.1),
                BindingMode::ReadWrite => {
                    let (r#struct, function) =
                        resource::binding_resource_read_write_printer(resource, map.1);
                    module.with_struct(r#struct);
                    function
                }
                BindingMode::ReadOnce => unreachable!(),
            };
            module.with_function(function);
        });
    });

    collection
        .components
        .into_iter()
        .for_each(|(component, maps)| {
            maps.into_iter().for_each(|map| {
                let (observers, filters) = map.1;
                if observers.is_empty() {
                    return;
                }
                let function = match map.0 {
                    BindingMode::Read => {
                        component::binding_component_read_printer(&component, observers, filters)
                    }
                    BindingMode::Write => {
                        component::binding_component_write_printer(&component, observers, filters)
                    }
                    BindingMode::ReadWrite => {
                        let (r#struct, function) = component::binding_component_read_write_printer(
                            &component, observers, filters,
                        );
                        module.with_struct(r#struct);
                        function
                    }
                    BindingMode::ReadOnce => unreachable!(),
                };
                module.with_function(function);
            });
        });

    module
}

pub struct Observer<'a> {
    target: &'a str,
    id: String,
    set: String,
    get: String,
}

#[derive(Default)]
struct ObserverCollection<'a> {
    resources: HashMap<String, HashMap<BindingMode, Vec<Observer<'a>>>>,
    components: HashMap<String, HashMap<BindingMode, (Vec<Observer<'a>>, Filters)>>,
}

fn prepare_filters(current: &Observer, observers: &[Observer]) -> Filters {
    let id = &current.id;

    let mut filters = Filters::default();
    filters.with(Filter::With(id.to_string()));

    if observers.len() == 1 {
        return filters;
    }

    observers
        .iter()
        .filter(|observer| observer.id != *id)
        .for_each(|observer| {
            let id = &observer.id;
            filters.with(Filter::Without(id.to_string()));
        });

    filters
}
