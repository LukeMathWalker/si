load(
    "@prelude-si//:macros.bzl",
    "alias",
    "export_file",
    "pnpm_lock",
    "workspace_node_modules",
)

alias(
    name = "council",
    actual = "//bin/council:council",
)

alias(
    name = "module-index",
    actual = "//bin/module-index:module-index",
)

alias(
    name = "pinga",
    actual = "//bin/pinga:pinga",
)

alias(
    name = "sdf",
    actual = "//bin/sdf:sdf",
)

alias(
    name = "veritech",
    actual = "//bin/veritech:veritech",
)

alias(
    name = "prepare",
    actual = "//component/deploy:prepare",
)

alias(
    name = "down",
    actual = "//component/deploy:down",
)

alias(
    name = "web",
    actual = "//app/web:dev",
)

alias(
    name = "auth-portal",
    actual = "//app/auth-portal:dev",
)

alias(
    name = "auth-api",
    actual = "//bin/auth-api:dev",
)

export_file(
    name = "package.json",
)

export_file(
    name = "pnpm-workspace.yaml",
)

pnpm_lock(
    name = "pnpm-lock.yaml",
    packages = [
        "//app/auth-portal:package.json",
        "//app/web:package.json",
        "//bin/auth-api:package.json",
        "//bin/lang-js:package.json",
        "//lib/eslint-config:package.json",
        "//lib/ts-lib:package.json",
        "//lib/tsconfig:package.json",
        "//lib/vue-lib:package.json",
    ],
)

workspace_node_modules(
    name = "node_modules",
)
