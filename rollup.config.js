import typescript from "@rollup/plugin-typescript";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import rollupJson from "rollup-plugin-json";
import commonjs from "@rollup/plugin-commonjs";
import execute from "rollup-plugin-execute";
import path from "path";
import { wasm } from '@rollup/plugin-wasm';

const isProd = process.env.BUILD === "production";

const currentDir = path.dirname(__filename);

const banner = `/*
THIS IS A GENERATED/BUNDLED FILE BY ROLLUP
if you want to view the source visit the plugins github repository
*/
`;

export default {
  input: "ts/main.ts",
  output: {
    dir: "./",
    sourcemap: "none",
    sourcemapExcludeSources: isProd,
    format: "cjs",
    exports: "default",
    banner,
  },
  external: ["obsidian", "obsidian-deer-toolbox-wasm"],
  plugins: [
    // execute([
    //   `mkdir -p ${currentDir}/build`,
    //   `cp ${currentDir}/manifest.json ${currentDir}/build/manifest.json`,
    //   `cp ${currentDir}/styles.css ${currentDir}/build/styles.css`,
    // ]),
    typescript(),
    wasm(),
    nodeResolve({ browser: true }),
    rollupJson(),
    commonjs(),
  ],
  onwarn(warning, warn) {
    // workaround to prevent rollup build error
    if (
      warning.code === "EVAL" &&
      /.*\/node_modules\/file-type\/.*/.test(warning.loc.file)
    ) {
      return;
    }
    warn(warning);
  },
};
