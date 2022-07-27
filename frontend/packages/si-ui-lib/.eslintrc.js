module.exports = {
  root: true,
  env: {
    node: true,
  },
  plugins: ["vue", "@typescript-eslint"],
  parser: "vue-eslint-parser",
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
  extends: [
    "plugin:vue/vue3-recommended",
    "eslint:recommended",
    "@vue/typescript/recommended",
    "@vue/prettier",
  ],
  rules: {
    camelcase: "off",
    "no-console": "off",
    "no-debugger": "off",
    "no-alert": "error",
    "no-unused-vars": "off", // Causes issues with ts enums
    "@typescript-eslint/no-unused-vars": [
      "warn",
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
      },
    ],
    "vue/script-setup-uses-vars": "error",
    "@typescript-eslint/ban-ts-comment": "off",
    "vue/multi-word-component-names": "off",
  },
  
};
