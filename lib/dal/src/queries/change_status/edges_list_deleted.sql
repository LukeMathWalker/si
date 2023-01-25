SELECT row_to_json(e.*) AS object
FROM edges e
WHERE e.id IN (SELECT id
               FROM edges
               WHERE visibility_change_set_pk = ident_nil_v1()
                 AND visibility_deleted_at IS NULL
                 AND in_tenancy_v1($1,
                                   tenancy_billing_account_pks,
                                   tenancy_organization_pks,
                                   tenancy_workspace_ids))

  AND visibility_change_set_pk = $2
  AND visibility_deleted_at IS NOT NULL

  AND in_tenancy_v1($1,
                    tenancy_billing_account_pks,
                    tenancy_organization_pks,
                    tenancy_workspace_ids)
ORDER BY e.id DESC
