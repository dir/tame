#!/usr/bin/env node

import { check, fix } from "./index.js";

function main() {
  const command = process.argv[2];
  const path = process.argv[3] || process.cwd();

  try {
    switch (command) {
      case "check":
        const hasCatalog = check(path);
        if (!hasCatalog) {
          console.log("No catalog entries found in workspace");
          process.exit(1);
        }
        break;
      case "fix":
        fix(path);
        break;
      default:
        console.log("Usage: tame <check|fix> [path]");
        process.exit(1);
    }
  } catch (error) {
    console.error("Error:", error.message);
    process.exit(1);
  }
}

main();
