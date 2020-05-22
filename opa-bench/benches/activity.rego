package test

allow {
    input.operation.connect
    input.auth_id.identity == "auth_id"
    input.client_id == "client_id"
}
