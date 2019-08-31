# COM

[![Build Status](https://dev.azure.com/microsoft-rust/com-rs/_apis/build/status/microsoft.com-rs?branchName=master)](https://dev.azure.com/microsoft-rust/com-rs/_build/latest?definitionId=1&branchName=master)

A one stop shop for all things related to [COM](https://docs.microsoft.com/en-us/windows/win32/com/component-object-model--com--portal) programming in Rust.

## Features

### comout_trust_callee

If this feature is set, ComOut will trust that the callee is initializing all the `[out]` parameters on success and will make them available as "safe to unwrap".