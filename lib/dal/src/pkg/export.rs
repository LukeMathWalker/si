use chrono::Utc;
use std::collections::{hash_map::Entry, HashMap};
use strum::IntoEnumIterator;
use telemetry::prelude::*;

use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, ChangeSetSpec, FuncArgumentSpec,
    FuncSpec, FuncSpecData, LeafFunctionSpec, MapKeyFuncSpec, PkgSpec, PropSpec, PropSpecBuilder,
    PropSpecKind, SchemaSpec, SchemaSpecData, SchemaVariantSpec, SchemaVariantSpecBuilder,
    SchemaVariantSpecComponentType, SchemaVariantSpecData, SchemaVariantSpecPropRoot, SiPkg,
    SiPkgKind, SiPropFuncSpec, SiPropFuncSpecKind, SocketSpec, SocketSpecData, SocketSpecKind,
    SpecError, ValidationSpec, ValidationSpecKind,
};

use crate::schema::variant::definition::SchemaVariantDefinition;
use crate::ChangeSetPk;
use crate::{
    func::{argument::FuncArgument, backend::validation::FuncBackendValidationArgs},
    prop_tree::{PropTree, PropTreeNode},
    socket::SocketKind,
    validation::Validation,
    ActionPrototype, ActionPrototypeContext, AttributeContextBuilder, AttributePrototype,
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, ChangeSet, ComponentType,
    DalContext, ExternalProvider, ExternalProviderId, Func, FuncId, InternalProvider,
    InternalProviderId, LeafInputLocation, LeafKind, Prop, PropId, PropKind, Schema, SchemaId,
    SchemaVariant, SchemaVariantError, SchemaVariantId, Socket, StandardModel, ValidationPrototype,
};

use super::{PkgError, PkgResult};

type ChangeSetMap = HashMap<FuncId, FuncSpec>;

#[derive(Debug)]
struct FuncSpecMap(HashMap<ChangeSetPk, ChangeSetMap>);

impl FuncSpecMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[allow(dead_code)]
    pub fn get_in_changeset_only(
        &self,
        change_set_pk: Option<ChangeSetPk>,
        func_id: FuncId,
    ) -> Option<&FuncSpec> {
        match self.0.get(&change_set_pk.unwrap_or(ChangeSetPk::NONE)) {
            Some(change_set_map) => change_set_map.get(&func_id),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, change_set_pk: Option<ChangeSetPk>, func_id: FuncId) -> Option<&FuncSpec> {
        match self.0.get(&change_set_pk.unwrap_or(ChangeSetPk::NONE)) {
            Some(change_set_map) => change_set_map.get(&func_id).or_else(|| {
                self.0
                    .get(&ChangeSetPk::NONE)
                    .and_then(|funcs| funcs.get(&func_id))
            }),
            None => self
                .0
                .get(&ChangeSetPk::NONE)
                .and_then(|funcs| funcs.get(&func_id)),
        }
    }

    pub fn init_change_set_map(&mut self, change_set_pk: Option<ChangeSetPk>) {
        self.0
            .entry(change_set_pk.unwrap_or(ChangeSetPk::NONE))
            .or_default();
    }

    pub fn contains_func(&self, change_set_pk: Option<ChangeSetPk>, func_id: FuncId) -> bool {
        match self.0.get(&change_set_pk.unwrap_or(ChangeSetPk::NONE)) {
            None => false,
            Some(inner_map) => inner_map.contains_key(&func_id),
        }
    }

    pub fn insert(
        &mut self,
        change_set_pk: Option<ChangeSetPk>,
        func_id: FuncId,
        spec: FuncSpec,
    ) -> Option<FuncSpec> {
        let change_set_pk = change_set_pk.unwrap_or(ChangeSetPk::NONE);
        self.0
            .entry(change_set_pk)
            .or_insert(HashMap::new())
            .insert(func_id, spec)
    }
}

pub struct PkgExporter {
    name: String,
    version: String,
    description: Option<String>,
    kind: SiPkgKind,
    created_by: String,
    schema_ids: Option<Vec<SchemaId>>,
    func_map: FuncSpecMap,
    is_workspace_export: bool,
}

fn std_model_change_set_matches<StdModel: StandardModel>(
    change_set_pk: Option<ChangeSetPk>,
    standard_model_thing: &StdModel,
) -> bool {
    match change_set_pk {
        None => true,
        Some(change_set_pk) => standard_model_thing.visibility().change_set_pk == change_set_pk,
    }
}

fn change_set_matches(
    current_change_set_pk: Option<ChangeSetPk>,
    object_change_set_pk: ChangeSetPk,
) -> bool {
    match current_change_set_pk {
        None => true,
        Some(current_change_set_pk) => object_change_set_pk == current_change_set_pk,
    }
}

impl PkgExporter {
    pub fn new_module_exporter(
        name: impl Into<String>,
        version: impl Into<String>,
        description: Option<impl Into<String>>,
        created_by: impl Into<String>,
        schema_ids: Vec<SchemaId>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.map(Into::into),
            kind: SiPkgKind::Module,
            created_by: created_by.into(),
            schema_ids: Some(schema_ids),
            func_map: FuncSpecMap::new(),
            is_workspace_export: false,
        }
    }

    pub fn new_workspace_exporter(name: impl Into<String>, created_by: impl Into<String>) -> Self {
        let version = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();
        let description = "workspace backup";

        Self {
            name: name.into(),
            version,
            description: Some(description.into()),
            kind: SiPkgKind::WorkspaceBackup,
            created_by: created_by.into(),
            schema_ids: None,
            func_map: FuncSpecMap::new(),
            is_workspace_export: true,
        }
    }

    pub async fn export_as_bytes(&mut self, ctx: &DalContext) -> PkgResult<Vec<u8>> {
        match self.kind {
            SiPkgKind::Module => info!("Building module package"),
            SiPkgKind::WorkspaceBackup => info!("Building workspace backup package"),
        }

        let pkg = self.export(ctx).await?;

        info!("Exporting as bytes");

        Ok(pkg.write_to_bytes()?)
    }

    async fn export_schema(
        &mut self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        schema: &Schema,
    ) -> PkgResult<(SchemaSpec, Vec<FuncSpec>)> {
        let variants = schema.variants(ctx).await?;
        let mut funcs = vec![];

        let mut schema_spec_builder = SchemaSpec::builder();
        schema_spec_builder.name(schema.name());
        if self.is_workspace_export {
            schema_spec_builder.unique_id(schema.id().to_string());
        }

        let in_change_set = std_model_change_set_matches(change_set_pk, schema);
        let is_deleted = schema.visibility().is_deleted();

        if in_change_set && is_deleted {
            schema_spec_builder.deleted(true);
        } else if in_change_set {
            let mut data_builder = SchemaSpecData::builder();
            data_builder.name(schema.name());
            data_builder.ui_hidden(schema.ui_hidden());
            let schema_ui_menu = schema.ui_menus(ctx).await?.pop().ok_or_else(|| {
                PkgError::StandardModelMissingBelongsTo(
                    "schema_ui_menu_belongs_to_schema",
                    "schema",
                    (*schema.id()).to_string(),
                )
            })?;
            data_builder.category(schema_ui_menu.category());
            data_builder.category_name(schema_ui_menu.name());
            schema_spec_builder.data(data_builder.build()?);
        }

        for variant in &variants {
            let related_funcs = SchemaVariant::all_funcs(ctx, *variant.id()).await?;

            for func in &related_funcs {
                if !self.func_map.contains_func(change_set_pk, *func.id()) {
                    if let Some(func_spec) = self.export_func(ctx, change_set_pk, func).await? {
                        self.func_map
                            .insert(change_set_pk, *func.id(), func_spec.clone());
                        funcs.push(func_spec);
                    }
                }
            }

            if !is_deleted {
                let variant_spec = self.export_variant(ctx, change_set_pk, variant).await?;
                schema_spec_builder.variant(variant_spec);
            }
        }

        let schema_spec = schema_spec_builder.build()?;

        Ok((schema_spec, funcs))
    }

    async fn export_variant(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant: &SchemaVariant,
    ) -> PkgResult<SchemaVariantSpec> {
        let mut variant_spec_builder = SchemaVariantSpec::builder();
        variant_spec_builder.name(variant.name());

        let schema_variant_definition =
            SchemaVariantDefinition::get_by_schema_variant_id(ctx, variant.id())
                .await?
                .ok_or(PkgError::MissingSchemaVariantDefinition(*variant.id()))?;

        if std_model_change_set_matches(change_set_pk, variant)
            || std_model_change_set_matches(change_set_pk, &schema_variant_definition)
        {
            if std_model_change_set_matches(change_set_pk, variant)
                && variant.visibility().is_deleted()
            {
                variant_spec_builder.deleted(true);
            }

            let mut data_builder = SchemaVariantSpecData::builder();

            data_builder.name(variant.name());

            if let Some(color_str) = variant.color(ctx).await? {
                data_builder.color(color_str);
            };
            if let Some(link) = variant.link() {
                data_builder.try_link(link)?;
            }

            data_builder.component_type(get_component_type(ctx, variant).await?);

            let asset_func_unique_id = self
                .func_map
                .get(change_set_pk, schema_variant_definition.func_id())
                .ok_or(PkgError::MissingExportedFunc(
                    schema_variant_definition.func_id(),
                ))?
                .unique_id
                .to_owned();

            data_builder.func_unique_id(asset_func_unique_id);

            variant_spec_builder.data(data_builder.build()?);
        }

        self.export_prop_tree(
            ctx,
            change_set_pk,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::Domain,
        )
        .await?;

        self.export_prop_tree(
            ctx,
            change_set_pk,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::ResourceValue,
        )
        .await?;

        self.export_leaf_funcs(ctx, change_set_pk, *variant.id())
            .await?
            .drain(..)
            .for_each(|leaf_func_spec| {
                variant_spec_builder.leaf_function(leaf_func_spec);
            });

        self.export_sockets(ctx, change_set_pk, *variant.id())
            .await?
            .drain(..)
            .for_each(|socket_spec| {
                variant_spec_builder.socket(socket_spec);
            });

        self.export_action_funcs(ctx, change_set_pk, *variant.id())
            .await?
            .drain(..)
            .for_each(|action_func_spec| {
                variant_spec_builder.action_func(action_func_spec);
            });

        self.export_si_prop_funcs(ctx, change_set_pk, variant)
            .await?
            .drain(..)
            .for_each(|si_prop_func_spec| {
                variant_spec_builder.si_prop_func(si_prop_func_spec);
            });

        let variant_spec = variant_spec_builder.build()?;

        Ok(variant_spec)
    }

    async fn export_si_prop_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant: &SchemaVariant,
    ) -> PkgResult<Vec<SiPropFuncSpec>> {
        let mut specs = vec![];

        for kind in SiPropFuncSpecKind::iter() {
            let prop = variant.find_prop(ctx, &kind.prop_path()).await?;

            let context = AttributeContextBuilder::new()
                .set_prop_id(*prop.id())
                .to_context()?;

            if let Some(prototype) =
                AttributePrototype::find_for_context_and_key(ctx, context, &None)
                    .await?
                    .pop()
            {
                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &prototype)
                    .await?
                {
                    let mut builder = SiPropFuncSpec::builder();
                    builder
                        .deleted(prototype.visibility().is_deleted())
                        .func_unique_id(func_unique_id)
                        .kind(kind);

                    if self.is_workspace_export {
                        builder.unique_id(prototype.id().to_string());
                    }

                    inputs.drain(..).for_each(|input| {
                        builder.input(input);
                    });

                    specs.push(builder.build()?);
                }
            }
        }

        Ok(specs)
    }

    async fn export_leaf_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<LeafFunctionSpec>> {
        let mut specs = vec![];

        for leaf_kind in LeafKind::iter() {
            for (prototype, leaf_func) in
                SchemaVariant::find_leaf_item_functions(ctx, variant_id, leaf_kind).await?
            {
                if !std_model_change_set_matches(change_set_pk, &prototype) {
                    continue;
                }

                let func_spec = self
                    .func_map
                    .get(change_set_pk, *leaf_func.id())
                    .ok_or(PkgError::MissingExportedFunc(*leaf_func.id()))?;

                let mut inputs = vec![];
                for arg in FuncArgument::list_for_func(ctx, *leaf_func.id()).await? {
                    if arg.visibility().is_deleted() {
                        continue;
                    }

                    inputs.push(
                        LeafInputLocation::maybe_from_arg_name(arg.name())
                            .ok_or(SpecError::LeafInputLocationConversionError(
                                arg.name().into(),
                            ))?
                            .into(),
                    );
                }

                let mut builder = LeafFunctionSpec::builder();
                if self.is_workspace_export {
                    builder.unique_id(prototype.id().to_string());
                }

                specs.push(
                    builder
                        .func_unique_id(&func_spec.unique_id)
                        .leaf_kind(leaf_kind)
                        .inputs(inputs)
                        .deleted(prototype.visibility().is_deleted())
                        .build()?,
                );
            }
        }

        Ok(specs)
    }

    async fn export_sockets(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<SocketSpec>> {
        let mut specs = vec![];

        for input_socket_ip in
            InternalProvider::list_explicit_for_schema_variant(ctx, variant_id).await?
        {
            let socket = Socket::find_for_internal_provider(ctx, *input_socket_ip.id())
                .await?
                .pop()
                .ok_or(PkgError::ExplicitInternalProviderMissingSocket(
                    *input_socket_ip.id(),
                ))?;

            if let SocketKind::Frame = socket.kind() {
                continue;
            }

            let mut socket_spec_builder = SocketSpec::builder();
            socket_spec_builder.name(input_socket_ip.name());

            if self.is_workspace_export {
                socket_spec_builder.unique_id(input_socket_ip.id().to_string());
            }

            let mut data_builder = SocketSpecData::builder();

            data_builder
                .name(input_socket_ip.name())
                .kind(SocketSpecKind::Input)
                .arity(socket.arity())
                .ui_hidden(socket.ui_hidden());

            let mut has_custom_func = false;
            if let Some(attr_proto_id) = input_socket_ip.attribute_prototype_id() {
                let proto = AttributePrototype::get_by_id(ctx, attr_proto_id)
                    .await?
                    .ok_or(PkgError::MissingAttributePrototypeForInputSocket(
                        *attr_proto_id,
                        *input_socket_ip.id(),
                    ))?;

                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &proto)
                    .await?
                {
                    has_custom_func = true;
                    data_builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        socket_spec_builder.input(input);
                    });
                }
            }

            if std_model_change_set_matches(change_set_pk, &socket) || has_custom_func {
                socket_spec_builder.data(data_builder.build()?);
            }

            specs.push(socket_spec_builder.build()?);
        }

        for output_socket_ep in ExternalProvider::list_for_schema_variant(ctx, variant_id).await? {
            let socket = Socket::find_for_external_provider(ctx, *output_socket_ep.id())
                .await?
                .pop()
                .ok_or(PkgError::ExternalProviderMissingSocket(
                    *output_socket_ep.id(),
                ))?;

            if let SocketKind::Frame = socket.kind() {
                continue;
            }

            let mut socket_spec_builder = SocketSpec::builder();
            socket_spec_builder.name(output_socket_ep.name());

            if self.is_workspace_export {
                socket_spec_builder.unique_id(output_socket_ep.id().to_string());
            }

            let mut data_builder = SocketSpecData::builder();
            data_builder
                .name(output_socket_ep.name())
                .kind(SocketSpecKind::Output)
                .arity(socket.arity())
                .ui_hidden(socket.ui_hidden());

            let mut has_custom_func = false;
            if let Some(attr_proto_id) = output_socket_ep.attribute_prototype_id() {
                let proto = AttributePrototype::get_by_id(ctx, attr_proto_id)
                    .await?
                    .ok_or(PkgError::MissingAttributePrototypeForOutputSocket(
                        *attr_proto_id,
                        *output_socket_ep.id(),
                    ))?;

                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &proto)
                    .await?
                {
                    has_custom_func = true;
                    data_builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        socket_spec_builder.input(input);
                    });
                }
            }

            if std_model_change_set_matches(change_set_pk, &socket) || has_custom_func {
                socket_spec_builder.data(data_builder.build()?);
            }

            specs.push(socket_spec_builder.build()?);
        }

        Ok(specs)
    }

    async fn export_action_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<ActionFuncSpec>> {
        let mut specs = vec![];

        let action_prototypes = ActionPrototype::find_for_context(
            ctx,
            ActionPrototypeContext {
                schema_variant_id: variant_id,
            },
        )
        .await?;

        for action_proto in action_prototypes {
            if !std_model_change_set_matches(change_set_pk, &action_proto) {
                continue;
            }

            let func_spec = self
                .func_map
                .get(change_set_pk, action_proto.func_id())
                .ok_or(PkgError::MissingExportedFunc(action_proto.func_id()))?;

            let mut builder = ActionFuncSpec::builder();

            if self.is_workspace_export {
                builder.unique_id(action_proto.id().to_string());
            }

            specs.push(
                builder
                    .kind(action_proto.kind())
                    .func_unique_id(&func_spec.unique_id)
                    .deleted(action_proto.visibility().is_deleted())
                    .build()?,
            )
        }

        Ok(specs)
    }

    async fn export_prop_tree(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant: &SchemaVariant,
        variant_spec: &mut SchemaVariantSpecBuilder,
        prop_root: SchemaVariantSpecPropRoot,
    ) -> PkgResult<()> {
        let mut prop_tree = PropTree::new(ctx, true, Some(vec![*variant.id()]), None).await?;
        let root_tree_node = prop_tree
            .root_props
            .pop()
            .ok_or_else(|| PkgError::prop_tree_invalid("root prop not found"))?;
        if !prop_tree.root_props.is_empty() {
            return Err(PkgError::prop_tree_invalid(
                "prop tree contained multiple root props",
            ));
        }
        let prop_root_tree_node = match root_tree_node.children.into_iter().find(|tree_node| {
            match prop_root {
                SchemaVariantSpecPropRoot::Domain => {
                    tree_node.name == "domain" && tree_node.path == "/root/"
                }
                SchemaVariantSpecPropRoot::ResourceValue => {
                    tree_node.name == "resource_value" && tree_node.path == "/root/"
                }
                SchemaVariantSpecPropRoot::Secrets => {
                    tree_node.name == "secrets" && tree_node.path == "/root/"
                }
                SchemaVariantSpecPropRoot::SecretDefinition => {
                    tree_node.name == "secret_definition" && tree_node.path == "/root/"
                }
            }
        }) {
            Some(root_tree_node) => root_tree_node,
            None => {
                if matches!(prop_root, SchemaVariantSpecPropRoot::Domain) {
                    return Err(PkgError::prop_tree_invalid("domain prop not found"));
                } else {
                    warn!("/root/resource_value prop not found, if value prop PR has merged, this should be an error not a warning.");
                    return Ok(());
                }
            }
        };

        #[derive(Debug)]
        struct TraversalStackEntry {
            builder: PropSpecBuilder,
            prop_id: PropId,
            parent_prop_id: Option<PropId>,
            inside_map_or_array: bool,
        }

        let mut stack: Vec<(PropTreeNode, Option<PropId>, bool)> = Vec::new();
        for child_tree_node in prop_root_tree_node.children {
            stack.push((child_tree_node, None, false));
        }

        let mut traversal_stack: Vec<TraversalStackEntry> = Vec::new();

        while let Some((tree_node, parent_prop_id, inside_map_or_array)) = stack.pop() {
            let prop_id = tree_node.prop_id;
            let mut builder = PropSpec::builder();

            if !change_set_matches(change_set_pk, tree_node.visibility_change_set_pk) {
                builder.has_data(false);
            }

            if self.is_workspace_export {
                builder.unique_id(prop_id);
            }

            builder
                .name(tree_node.name)
                .kind(match tree_node.kind {
                    PropKind::Array => PropSpecKind::Array,
                    PropKind::Boolean => PropSpecKind::Boolean,
                    PropKind::Integer => PropSpecKind::Number,
                    PropKind::Object => PropSpecKind::Object,
                    PropKind::String => PropSpecKind::String,
                    PropKind::Map => PropSpecKind::Map,
                })
                .hidden(tree_node.hidden)
                .widget_kind(tree_node.widget_kind)
                .widget_options(tree_node.widget_options);

            if let Some(doc_link) = tree_node.doc_link {
                builder.try_doc_link(doc_link.as_str())?;
            }

            traversal_stack.push(TraversalStackEntry {
                builder,
                prop_id,
                parent_prop_id,
                inside_map_or_array,
            });

            for child_tree_node in tree_node.children {
                stack.push((
                    child_tree_node,
                    Some(prop_id),
                    matches!(tree_node.kind, PropKind::Array | PropKind::Map)
                        || inside_map_or_array,
                ));
            }
        }

        let mut prop_children_map: HashMap<PropId, Vec<(PropSpec, PropId)>> = HashMap::new();

        while let Some(mut entry) = traversal_stack.pop() {
            let mut maybe_type_prop_id: Option<PropId> = None;

            if let Some(mut prop_children) = prop_children_map.remove(&entry.prop_id) {
                match entry.builder.get_kind() {
                    Some(kind) => match kind {
                        PropSpecKind::Object => {
                            entry.builder.entries(
                                prop_children
                                    .iter()
                                    .map(|(prop_spec, _)| prop_spec.to_owned())
                                    .collect(),
                            );
                        }
                        PropSpecKind::Map | PropSpecKind::Array => {
                            let (type_prop, type_prop_id) =
                                prop_children.pop().ok_or_else(|| {
                                    PkgError::prop_spec_children_invalid(format!(
                                        "found no child for map/array for prop id {}",
                                        entry.prop_id,
                                    ))
                                })?;
                            if !prop_children.is_empty() {
                                return Err(PkgError::prop_spec_children_invalid(format!(
                                    "found multiple children for map/array for prop id {}",
                                    entry.prop_id,
                                )));
                            }
                            entry.builder.type_prop(type_prop);
                            maybe_type_prop_id = Some(type_prop_id);
                        }
                        PropSpecKind::String | PropSpecKind::Number | PropSpecKind::Boolean => {
                            return Err(PkgError::prop_spec_children_invalid(format!(
                                "primitve prop type should have no children for prop id {}",
                                entry.prop_id,
                            )));
                        }
                    },
                    None => {
                        return Err(SpecError::UninitializedField("kind").into());
                    }
                };
            }

            if matches!(entry.builder.get_kind(), Some(PropSpecKind::Map)) {
                if let Some(type_prop_id) = maybe_type_prop_id {
                    let context = AttributeContextBuilder::new()
                        .set_prop_id(type_prop_id)
                        .to_context()?;

                    for proto in AttributePrototype::list_for_context(ctx, context).await? {
                        if let Some(key) = proto.key() {
                            if let Some((func_unique_id, mut inputs)) = self
                                .export_input_func_and_arguments(ctx, change_set_pk, &proto)
                                .await?
                            {
                                let mut map_key_func_builder = MapKeyFuncSpec::builder();
                                map_key_func_builder.key(key);
                                map_key_func_builder.func_unique_id(func_unique_id);
                                inputs.drain(..).for_each(|input| {
                                    map_key_func_builder.input(input);
                                });
                                entry.builder.map_key_func(map_key_func_builder.build()?);
                            }
                        }
                    }
                }
            }

            // TODO: if we get funcs here but we also got map_key_funcs above, that's a sign of a
            // TODO: misconfigured set of attribute prototypes. check and error
            let context = AttributeContextBuilder::new()
                .set_prop_id(entry.prop_id)
                .to_context()?;

            if let Some(prototype) =
                AttributePrototype::find_for_context_and_key(ctx, context, &None)
                    .await?
                    .pop()
            {
                if std_model_change_set_matches(change_set_pk, &prototype) {
                    entry.builder.has_data(true);
                }

                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &prototype)
                    .await?
                {
                    entry.builder.has_data(true);

                    entry.builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        entry.builder.input(input);
                    });
                }
            }

            // TODO: handle default values for complex types. We also cannot set default values for
            // children of arrays and maps, at any depth (currently), since that requires tracking the
            // key or index
            if matches!(
                entry.builder.get_kind(),
                Some(PropSpecKind::String)
                    | Some(PropSpecKind::Number)
                    | Some(PropSpecKind::Boolean)
            ) && !entry.inside_map_or_array
            {
                if let Some(av) = AttributeValue::find_for_context(ctx, context.into()).await? {
                    if let Some(default_value) = av.get_value(ctx).await? {
                        entry.builder.has_data(true);
                        entry.builder.default_value(default_value);
                    }
                }
            }

            for validation in self
                .export_validations_for_prop(ctx, change_set_pk, entry.prop_id)
                .await?
            {
                entry.builder.validation(validation);
            }

            let prop_spec = entry.builder.build()?;

            match entry.parent_prop_id {
                None => {
                    variant_spec.prop(prop_root, prop_spec);
                }
                Some(parent_prop_id) => {
                    match prop_children_map.entry(parent_prop_id) {
                        Entry::Occupied(mut occupied) => {
                            occupied.get_mut().push((prop_spec, entry.prop_id));
                        }
                        Entry::Vacant(vacant) => {
                            vacant.insert(vec![(prop_spec, entry.prop_id)]);
                        }
                    };
                }
            };
        }

        Ok(())
    }

    async fn export_input_func_and_arguments(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        proto: &AttributePrototype,
    ) -> PkgResult<Option<(String, Vec<AttrFuncInputSpec>)>> {
        let proto_func = Func::get_by_id(ctx, &proto.func_id()).await?.ok_or(
            PkgError::MissingAttributePrototypeFunc(*proto.id(), proto.func_id()),
        )?;

        let apas: Vec<AttributePrototypeArgument> =
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id())
                .await?
                .into_iter()
                .filter(|apa| std_model_change_set_matches(change_set_pk, apa))
                .collect();

        // If the prototype func is intrinsic and has no arguments, it's one that is created by default
        // and we don't have to track it in the package
        if apas.is_empty() && proto_func.is_intrinsic() {
            return Ok(None);
        }

        let mut inputs = vec![];

        for apa in &apas {
            let func_arg = FuncArgument::get_by_id(ctx, &apa.func_argument_id())
                .await?
                .ok_or(PkgError::AttributePrototypeArgumentMissingFuncArgument(
                    *apa.id(),
                    apa.func_argument_id(),
                ))?;
            let arg_name = func_arg.name();

            let mut builder = AttrFuncInputSpec::builder();
            if self.is_workspace_export {
                builder.unique_id(apa.id().to_string());
            }
            builder
                .name(arg_name)
                .deleted(apa.visibility().is_deleted());

            if apa.internal_provider_id() != InternalProviderId::NONE {
                let ip = InternalProvider::get_by_id(ctx, &apa.internal_provider_id())
                    .await?
                    .ok_or(PkgError::AttributePrototypeArgumentMissingInternalProvider(
                        *apa.id(),
                        apa.internal_provider_id(),
                    ))?;

                match *ip.prop_id() {
                    PropId::NONE => {
                        inputs.push(
                            builder
                                .name(arg_name)
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .socket_name(ip.name())
                                .build()?,
                        );
                    }
                    prop_id => {
                        let prop = Prop::get_by_id(ctx, &prop_id)
                            .await?
                            .ok_or(PkgError::InternalProviderMissingProp(*ip.id(), prop_id))?;

                        inputs.push(
                            builder
                                .name(arg_name)
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(prop.path())
                                .build()?,
                        );
                    }
                }
            } else if apa.external_provider_id() != ExternalProviderId::NONE {
                let ep = ExternalProvider::get_by_id(ctx, &apa.external_provider_id())
                    .await?
                    .ok_or(PkgError::AttributePrototypeArgumentMissingExternalProvider(
                        *apa.id(),
                        apa.external_provider_id(),
                    ))?;

                inputs.push(
                    builder
                        .name(arg_name)
                        .kind(AttrFuncInputSpecKind::OutputSocket)
                        .socket_name(ep.name())
                        .build()?,
                );
            }
        }

        let func_spec = self
            .func_map
            .get(change_set_pk, *proto_func.id())
            .ok_or(PkgError::MissingExportedFunc(*proto_func.id()))?;

        let func_unique_id = func_spec.unique_id.to_owned();

        Ok(Some((func_unique_id, inputs)))
    }

    async fn export_validations_for_prop(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        prop_id: PropId,
    ) -> PkgResult<Vec<ValidationSpec>> {
        let mut validation_specs = vec![];

        let validation_prototypes = ValidationPrototype::list_for_prop(ctx, prop_id).await?;

        for prototype in &validation_prototypes {
            if !std_model_change_set_matches(change_set_pk, prototype) {
                continue;
            }

            let mut spec_builder = ValidationSpec::builder();

            if self.is_workspace_export {
                spec_builder.unique_id(prototype.id().to_string());
            }

            if prototype.visibility().is_deleted() {
                spec_builder.deleted(true);
            }

            let args: Option<FuncBackendValidationArgs> =
                serde_json::from_value(prototype.args().clone())?;

            match args {
                Some(validation) => match validation.validation {
                    Validation::IntegerIsBetweenTwoIntegers {
                        lower_bound,
                        upper_bound,
                        ..
                    } => {
                        spec_builder.kind(ValidationSpecKind::IntegerIsBetweenTwoIntegers);
                        spec_builder.upper_bound(upper_bound);
                        spec_builder.lower_bound(lower_bound);
                    }
                    Validation::IntegerIsNotEmpty { .. } => {
                        spec_builder.kind(ValidationSpecKind::IntegerIsNotEmpty);
                    }
                    Validation::StringHasPrefix { expected, .. } => {
                        spec_builder.kind(ValidationSpecKind::StringHasPrefix);
                        spec_builder.expected_string(expected);
                    }
                    Validation::StringEquals { expected, .. } => {
                        spec_builder.kind(ValidationSpecKind::StringEquals);
                        spec_builder.expected_string(expected);
                    }
                    Validation::StringInStringArray {
                        expected,
                        display_expected,
                        ..
                    } => {
                        spec_builder.kind(ValidationSpecKind::StringInStringArray);
                        spec_builder.expected_string_array(expected);
                        spec_builder.display_expected(display_expected);
                    }
                    Validation::StringIsNotEmpty { .. } => {
                        spec_builder.kind(ValidationSpecKind::StringIsNotEmpty);
                    }
                    Validation::StringIsValidIpAddr { .. } => {
                        spec_builder.kind(ValidationSpecKind::StringIsValidIpAddr);
                    }
                    Validation::StringIsHexColor { .. } => {
                        spec_builder.kind(ValidationSpecKind::StringIsHexColor);
                    }
                },
                None => {
                    let func_spec = self
                        .func_map
                        .get(change_set_pk, prototype.func_id())
                        .ok_or(PkgError::MissingExportedFunc(prototype.func_id()))?;

                    spec_builder.kind(ValidationSpecKind::CustomValidation);
                    spec_builder.func_unique_id(&func_spec.unique_id);
                }
            }

            validation_specs.push(spec_builder.build()?);
        }

        Ok(validation_specs)
    }

    async fn export_func(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        func: &Func,
    ) -> PkgResult<Option<FuncSpec>> {
        let mut func_spec_builder = FuncSpec::builder();

        func_spec_builder.name(func.name());

        let in_change_set = std_model_change_set_matches(change_set_pk, func);

        if in_change_set && func.visibility().is_deleted() {
            func_spec_builder.deleted(true);
            if self.is_workspace_export {
                func_spec_builder.unique_id(func.id().to_string());
            } else {
                // These ids will be stable so long as the function is unchanged
                func_spec_builder.unique_id(func_spec_builder.gen_unique_id()?);
            }
            return Ok(Some(func_spec_builder.build()?));
        }

        if in_change_set {
            let mut data_builder = FuncSpecData::builder();

            data_builder.name(func.name());

            if let Some(display_name) = func.display_name() {
                data_builder.display_name(display_name);
            }

            if let Some(description) = func.description() {
                data_builder.description(description);
            }

            if let Some(link) = func.link() {
                data_builder.try_link(link)?;
            }
            // Should we package an empty func?
            data_builder.handler(func.handler().unwrap_or(""));
            data_builder.code_base64(func.code_base64().unwrap_or(""));

            data_builder.response_type(*func.backend_response_type());
            data_builder.backend_kind(*func.backend_kind());

            data_builder.hidden(func.hidden());

            func_spec_builder.data(data_builder.build()?);
        }

        if self.is_workspace_export {
            func_spec_builder.unique_id(func.id().to_string());
        } else {
            // These ids will be stable so long as the function is unchanged
            func_spec_builder.unique_id(func_spec_builder.gen_unique_id()?);
        }

        let args: Vec<FuncArgument> = FuncArgument::list_for_func(ctx, *func.id())
            .await?
            .into_iter()
            .filter(|f| std_model_change_set_matches(change_set_pk, f))
            .collect();

        for arg in &args {
            let mut arg_builder = FuncArgumentSpec::builder();

            if self.is_workspace_export {
                arg_builder.unique_id(arg.id().to_string());
            }

            func_spec_builder.argument(
                arg_builder
                    .name(arg.name())
                    .kind(*arg.kind())
                    .element_kind(arg.element_kind().cloned().map(|kind| kind.into()))
                    .deleted(arg.visibility().is_deleted())
                    .build()?,
            );
        }

        let func_spec = func_spec_builder.build()?;

        // If we have data, or change set specific arguments, we're valid for this changeset
        Ok(if func_spec.data.is_some() || !args.is_empty() {
            Some(func_spec)
        } else {
            None
        })
    }

    /// If change_set_pk is None, we export everything in the changeset without checking for
    /// differences from HEAD. Otherwise we attempt to only export the data specific to the
    /// requested change_set
    async fn export_change_set(
        &mut self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
    ) -> PkgResult<(Vec<FuncSpec>, Vec<SchemaSpec>)> {
        let mut func_specs = vec![];
        let mut schema_specs = vec![];

        let new_ctx = match change_set_pk {
            None => ctx.clone(),
            Some(change_set_pk) => {
                ctx.clone_with_new_visibility(ctx.visibility().to_change_set_deleted(change_set_pk))
            }
        };
        let ctx = &new_ctx;

        self.func_map.init_change_set_map(change_set_pk);

        // Intrinsic funcs should be immutable. They're not, but we don't provide any interfaces to
        // modify them via a the standard model. We only add them to the func map if the func map
        // is HEAD (or if we're doing a module export)
        if change_set_pk.unwrap_or(ChangeSetPk::NONE) == ChangeSetPk::NONE {
            for intrinsic in crate::func::intrinsics::IntrinsicFunc::iter() {
                let intrinsic_name = intrinsic.name();
                // We need a unique id for intrinsic funcs to refer to them in custom bindings (for example
                // mapping one prop to another via si:identity)
                let intrinsic_func = Func::find_by_name(ctx, intrinsic_name)
                    .await?
                    .ok_or(PkgError::MissingIntrinsicFunc(intrinsic_name.to_string()))?;

                let intrinsic_spec = intrinsic.to_spec()?;
                self.func_map
                    .insert(change_set_pk, *intrinsic_func.id(), intrinsic_spec.clone());

                func_specs.push(intrinsic_spec);
            }
        }

        // XXX: make this SQL query
        let schemas: Vec<Schema> = Schema::list(ctx)
            .await?
            .into_iter()
            .filter(|sv| {
                if let Some(schema_ids) = &self.schema_ids {
                    schema_ids.contains(sv.id())
                } else {
                    true
                }
            })
            .collect();

        for schema in &schemas {
            let (schema_spec, funcs) = self.export_schema(ctx, change_set_pk, schema).await?;

            func_specs.extend_from_slice(&funcs);
            schema_specs.push(schema_spec);
        }

        Ok((func_specs, schema_specs))
    }

    pub async fn export(&mut self, ctx: &DalContext) -> PkgResult<SiPkg> {
        let mut pkg_spec_builder = PkgSpec::builder();
        pkg_spec_builder
            .name(&self.name)
            .kind(self.kind)
            .version(&self.version)
            .created_by(&self.created_by);

        if let Some(workspace_pk) = ctx.tenancy().workspace_pk() {
            pkg_spec_builder.workspace_pk(workspace_pk.to_string());
        }

        if let Some(description) = &self.description {
            pkg_spec_builder.description(description);
        }

        match self.kind {
            SiPkgKind::Module => {
                let (funcs, schemas) = self.export_change_set(ctx, None).await?;
                pkg_spec_builder.funcs(funcs);
                pkg_spec_builder.schemas(schemas);
            }
            SiPkgKind::WorkspaceBackup => {
                let (funcs, schemas) = self.export_change_set(ctx, Some(ChangeSetPk::NONE)).await?;
                pkg_spec_builder.change_set(
                    ChangeSetSpec::builder()
                        .name("head")
                        .funcs(funcs)
                        .schemas(schemas)
                        .build()?,
                );
                pkg_spec_builder.default_change_set("head");

                for change_set in ChangeSet::list_open(ctx).await? {
                    let (funcs, schemas) = self.export_change_set(ctx, Some(change_set.pk)).await?;

                    pkg_spec_builder.change_set(
                        ChangeSetSpec::builder()
                            .name(&change_set.name)
                            .based_on_change_set("head")
                            .funcs(funcs)
                            .schemas(schemas)
                            .build()?,
                    );
                }
            }
        }

        let spec = pkg_spec_builder.build()?;
        let pkg = SiPkg::load_from_spec(spec)?;

        Ok(pkg)
    }
}

pub async fn get_component_type(
    ctx: &DalContext,
    variant: &SchemaVariant,
) -> Result<SchemaVariantSpecComponentType, PkgError> {
    let type_prop = variant.find_prop(ctx, &["root", "si", "type"]).await?;
    let type_context = AttributeReadContext {
        prop_id: Some(*type_prop.id()),
        ..Default::default()
    };

    let type_av = AttributeValue::find_for_context(ctx, type_context)
        .await?
        .ok_or(SchemaVariantError::AttributeValueNotFoundForContext(
            type_context,
        ))?;

    Ok(match type_av.get_value(ctx).await? {
        Some(type_value) => {
            let component_type: ComponentType = serde_json::from_value(type_value)?;
            component_type.into()
        }
        None => SchemaVariantSpecComponentType::default(),
    })
}
