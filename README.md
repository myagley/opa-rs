# Open Policy Agent
The [Open Policy Agent](https://www.openpolicyagent.org/docs/latest/) (OPA, pronounced “oh-pa”) is an open source, general-purpose policy engine that unifies policy enforcement across the stack.
OPA provides a high-level declarative language that let’s you specify policy as code and simple APIs to offload policy decision-making from your software.
You can use OPA to enforce policies in microservices, Kubernetes, CI/CD pipelines, API gateways, and more.

--------

This project is a rust integration for OPA.
Its primary integration point is via [Web Assembly](https://www.openpolicyagent.org/docs/latest/wasm/).

# Quickstart

This assumes you have the OPA [cli tool](https://www.openpolicyagent.org/docs/latest/#running-opa) on the path to compile Rego files to WASM.

To run the example:

```sh
$ RUST_LOG=debug cargo run --example eval -- -q 'data.example.allow' -p opa-wasm/examples/example.rego -i opa-wasm/examples/input.json
```

You should see the following output, indicating that `data.example.allow` query is defined for the `opa-wasm/examples/input.json` input.

```
result: {{}}
```

The result is a set of variable bindings where the query is defined.
In this case, the result is a set of size one, meaning that the query is defined for this input.
