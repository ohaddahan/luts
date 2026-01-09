import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import promise from "eslint-plugin-promise";
import {globalIgnores} from "eslint/config";

export default tseslint.config([
    globalIgnores(["dist", "target", "codama-ts-luts", ".anchor", "tests-backup"]),
    {
        files: ["**/*.{ts,tsx}"],
        extends: [js.configs.recommended, tseslint.configs.recommended],
        plugins: {
            promise,
        },
        languageOptions: {
            ecmaVersion: 2020,
            globals: globals.node,
        },
        linterOptions: {
            reportUnusedDisableDirectives: "off",
        },
        rules: {
            "@typescript-eslint/no-explicit-any": "off",
            "@typescript-eslint/ban-ts-comment": "off",
            "@typescript-eslint/no-unused-vars": [
                "warn",
                {
                    argsIgnorePattern: "^_",
                    caughtErrorsIgnorePattern: "^_",
                    varsIgnorePattern: "^_",
                },
            ],
            "promise/catch-or-return": "warn",
            "promise/no-nesting": "warn",
            "promise/no-promise-in-callback": "warn",
            "promise/no-return-wrap": "error",
            "promise/param-names": "error",
            "promise/valid-params": "error",
            "no-console": ["warn", {allow: ["warn", "error", "log"]}],
            "no-debugger": "warn",
            "prefer-const": "warn",
            "no-var": "error",
        },
    },
]);
