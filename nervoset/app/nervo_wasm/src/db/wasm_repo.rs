use rexie::*;

pub struct WasmRepo {
    pub db_name: String,
    pub store_name: String,

    rexie: Rexie,
}

impl WasmRepo {
    pub async fn init() -> WasmRepo {
        let db_name = String::from("nervo-labs");
        let store_name = String::from("nervo-store");

        // Create a new database
        let rexie = Rexie::builder(db_name.as_str())
            // Set the version of the database to 1.0
            .version(1)
            // Add an object store named `employees`
            .add_object_store(
                ObjectStore::new(store_name.as_str()), // Set the key path to `id`
                                                       //.key_path("id")
                                                       // Enable auto increment
                                                       //.auto_increment(true)
            )
            // Build the database
            .build()
            .await
            .expect("Failed to create REXie");

        Self {
            db_name,
            store_name,
            rexie,
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store_name = self.store_name.as_str();

        let tx = self
            .rexie
            .transaction(&[store_name], TransactionMode::ReadWrite)
            .unwrap();

        let store = tx.store(store_name).unwrap();

        // Convert it to `JsValue`
        let js_key = serde_wasm_bindgen::to_value(key).unwrap();

        // Add the employee to the store
        store
            .get(js_key)
            .await
            .unwrap()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).unwrap())
    }

    pub async fn put(&self, key: &str, value: &str) {
        let store_name = self.store_name.as_str();

        let tx = self
            .rexie
            .transaction(&[store_name], TransactionMode::ReadWrite)
            .unwrap();

        let store = tx.store(store_name).unwrap();

        let js_key = serde_wasm_bindgen::to_value(key).unwrap();
        let js_value = serde_wasm_bindgen::to_value(value).unwrap();

        store.add(&js_value, Some(&js_key)).await.unwrap();

        // Waits for the transaction to complete
        tx.done().await.unwrap();
    }
}
