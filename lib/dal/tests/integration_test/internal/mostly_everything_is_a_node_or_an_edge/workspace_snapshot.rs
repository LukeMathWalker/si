//! For all tests in this file, provide "SI_TEST_BUILTIN_SCHEMAS=none" as an environment variable.

use content_store::{ContentHash, Store};
use content_store_test::DalTestPgStore;
use dal::change_set_pointer::ChangeSetPointer;
use dal::component::ComponentKind;
use dal::workspace_snapshot::content_address::ContentAddress;
use dal::workspace_snapshot::node_weight::NodeWeight;
use dal::{DalContext, Schema, WorkspaceSnapshot};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let store = DalTestPgStore::new().await.expect("could not create store");
    let mut change_set = ChangeSetPointer::new(ctx, "main")
        .await
        .expect("could not create change set pointer");
    let mut snapshot = WorkspaceSnapshot::initial(ctx, &change_set, store)
        .await
        .expect("could not create workspace snapshot");

    let node_index = snapshot
        .add_node(
            NodeWeight::new_content(
                &change_set,
                change_set
                    .generate_ulid()
                    .expect("cannot generate ulid"),
                ContentAddress::Schema(ContentHash::from("sarah is making me watch all the fast and furious movies and it's simultaneously awesome and painful")),
            )
                .expect("could not create node weight"),
        )
        .expect("could not add node");

    snapshot.write(ctx).await.expect("could not write snapshot");
    change_set
        .update_pointer(ctx, snapshot.id())
        .await
        .expect("could not update pointer");

    let value = snapshot
        .attribute_value_view(node_index)
        .await
        .expect("could not generate attribute value view");
    dbg!(value);
}
