#!/usr/bin/env node

import { execa } from "execa";

async function main() {
  const isRelease = process.argv[2] === "release";

  // Construct the flags array
  const flags = [
    "build",
    "--platform",
    "-p",
    "tame",
    "--cargo-name",
    "tame",
    "native",
    "--js",
    "false",
  ];

  // Add release or dts flag based on the argument
  if (isRelease) {
    flags.push("--release");
  } else {
    flags.push("--dts", "../js/index.d.ts");
  }

  // Run napi build
  await execa("node_modules/.bin/napi", flags);

  // Run prettier on the .d.ts file if it was generated (non-release mode)
  if (!isRelease) {
    await execa("node_modules/.bin/prettier", ["--write", "js/index.d.ts"]);
  }

  // Create dist directory and copy files
  await execa("mkdir", ["-p", "js/dist"]);
  await execa("cp", ["js/index.js", "js/index.d.ts", "js/dist/"]);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
