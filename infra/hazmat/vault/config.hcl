#cluster_addr = "https://127.0.0.1:8201"
api_addr = "http://0.0.0.0:8200"
ui = true
disable_mlock = "true"

/*storage "raft" {
  path    = "/vault/file"
  node_id = "vault_1"
}*/
storage "file" {
  path    = "/vault/file"
}

listener "tcp" {
  address = "0.0.0.0:8200"
  tls_disable = true
}
