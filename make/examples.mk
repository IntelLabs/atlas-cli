.PHONY: examples setup-examples help-examples \
        example-model-single example-model-multi \
        example-dataset-single example-dataset-multi \
        example-model-dataset-workflow example-full-workflow \
        example-export-provenance example-complex-workflow \
        run-json run-cbor

# Example command variables
MODEL_SINGLE_CMD = model create \
	--paths=$(EXAMPLES_MODELS_DIR)/model.onnx \
	--ingredient-names="Main Model" \
	--name="Single Model Example" \
	--author-org="Test Org" \
	--author-name="Test Author" \
	--format=json \
	--print \
	--key=private.pem

MODEL_MULTI_CMD = model create \
	--paths=$(EXAMPLES_MODELS_DIR)/feature_extractor.onnx,$(EXAMPLES_MODELS_DIR)/classifier.onnx \
	--ingredient-names="Feature Extractor,Classifier" \
	--name="Multi-Component Model" \
	--author-org="Test Org" \
	--author-name="Test Author" \
	--format=json \
	--print \
	--key=private.pem

DATASET_SINGLE_CMD = dataset create \
	--paths=$(EXAMPLES_DATA_TEST_DIR)/data1.txt \
	--ingredient-names="Main Dataset" \
	--name="Single Dataset Example" \
	--author-org="Test Org" \
	--author-name="Test Author" \
	--format=json \
	--print \
	--key=private.pem

DATASET_MULTI_CMD = dataset create \
	--paths=$(EXAMPLES_DATA_TRAIN_DIR)/data1.txt,$(EXAMPLES_DATA_TEST_DIR)/data1.txt,$(EXAMPLES_DATA_VALIDATION_DIR)/data1.txt \
	--ingredient-names="Training Set,Test Set,Validation Set" \
	--name="Complete Dataset" \
	--author-org="Test Org" \
	--author-name="Test Author" \
	--format=json \
	--print \
	--key=private.pem

# Database export command
EXPORT_PROVENANCE_DB_CMD = manifest export \
    --id="urn:c2pa:123e4567-e89b-12d3-a456-426614174000" \
    --storage-type=database \
    --storage-url=$(DEFAULT_STORAGE_URL) \
    --format=json \
    --output=provenance.json

# Filesystem export command
EXPORT_PROVENANCE_FS_CMD = manifest export \
    --id="urn:c2pa:123e4567-e89b-12d3-a456-426614174000" \
    --storage-type=local-fs \
    --storage-url=$(DEFAULT_FILESYSTEM_PATH) \
    --format=json \
    --output=provenance.json

# Create example directory structure and files
setup-examples:
	$(MKDIR_CMD) $(EXAMPLES_MODELS_DIR)
	$(MKDIR_CMD) $(EXAMPLES_DATA_TRAIN_DIR)
	$(MKDIR_CMD) $(EXAMPLES_DATA_TEST_DIR)
	$(MKDIR_CMD) $(EXAMPLES_DATA_VALIDATION_DIR)
	$(MKDIR_CMD) $(EXAMPLES_RESULTS_DIR)
	$(TOUCH_CMD) $(EXAMPLES_MODELS_DIR)/model.onnx
	$(TOUCH_CMD) $(EXAMPLES_MODELS_DIR)/feature_extractor.onnx
	$(TOUCH_CMD) $(EXAMPLES_MODELS_DIR)/classifier.onnx
	$(TOUCH_CMD) $(EXAMPLES_DATA_TRAIN_DIR)/data1.txt
	$(TOUCH_CMD) $(EXAMPLES_DATA_TEST_DIR)/data1.txt
	$(TOUCH_CMD) $(EXAMPLES_DATA_VALIDATION_DIR)/data1.txt
	@echo "Example directory structure created."

# Run with different output formats
run-json:
	$(CARGO) run -- $(MODEL_SINGLE_CMD)

run-cbor:
	$(CARGO) run -- model create \
		--paths=$(EXAMPLES_MODELS_DIR)/model.onnx \
		--ingredient-names="Main Model" \
		--name="CBOR Example" \
		--format=cbor \
		--print

# Examples with single ingredients
example-model-single: generate-keys setup-examples
	@echo "Running single model example..."
	$(CARGO) run -- $(MODEL_SINGLE_CMD)

# Examples with multiple ingredients
example-model-multi: generate-keys setup-examples
	@echo "Running multi-model example..."
	$(CARGO) run -- $(MODEL_MULTI_CMD)

# Example with single dataset
example-dataset-single: generate-keys setup-examples
	@echo "Running single dataset example..."
	$(CARGO) run -- $(DATASET_SINGLE_CMD)

# Example with multiple dataset components
example-dataset-multi: generate-keys setup-examples
	@echo "Running multi-dataset example..."
	$(CARGO) run -- $(DATASET_MULTI_CMD)

# Example for creating and linking model and dataset
example-model-dataset-workflow: generate-keys setup-examples
	@echo "Creating dataset manifest..."
	$(CARGO) run -- dataset create \
		--paths=$(EXAMPLES_DATA_TRAIN_DIR)/data1.txt \
		--ingredient-names="Training Dataset" \
		--name="Training Dataset" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> dataset_output.txt
	@echo "Extracting dataset ID..."
	@DATASET_ID=$$(grep -o "ID: [^ ]*" dataset_output.txt | cut -d' ' -f2); \
	echo "Dataset ID: $$DATASET_ID"
	@echo "Creating model manifest..."
	$(CARGO) run -- model create \
		--paths=$(EXAMPLES_MODELS_DIR)/model.onnx \
		--ingredient-names="ONNX Model" \
		--name="Trained Model" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> model_output.txt
	@echo "Extracting model ID..."
	@MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	echo "Model ID: $$MODEL_ID"
	@echo "Linking model to dataset..."
	@DATASET_ID=$$(grep -o "ID: [^ ]*" dataset_output.txt | cut -d' ' -f2); \
	MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	$(CARGO) run -- model link-dataset \
		--model-id="$$MODEL_ID" \
		--dataset-id="$$DATASET_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL)
	@echo "Cleaning up temporary files..."
	@$(RM_CMD) dataset_output.txt model_output.txt

# Full workflow example with step-by-step guidance
example-full-workflow: generate-keys setup-examples
	@echo "=== STEP 1: Creating dataset manifest ==="
	$(CARGO) run -- dataset create \
		--paths=$(EXAMPLES_DATA_TRAIN_DIR)/data1.txt \
		--ingredient-names="Training Dataset" \
		--name="Training Dataset" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> dataset_output.txt
	
	@DATASET_ID=$$(grep -o "ID: [^ ]*" dataset_output.txt | cut -d' ' -f2); \
	echo "Dataset ID: $$DATASET_ID"
	
	@echo ""
	@echo "=== STEP 2: Creating model manifest ==="
	$(CARGO) run -- model create \
		--paths=$(EXAMPLES_MODELS_DIR)/model.onnx \
		--ingredient-names="ONNX Model" \
		--name="Trained Model" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> model_output.txt
	
	@MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	echo "Model ID: $$MODEL_ID"
	@echo ""
	@echo "=== STEP 3: Linking model to dataset ==="
	@DATASET_ID=$$(grep -o "ID: [^ ]*" dataset_output.txt | cut -d' ' -f2); \
	MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	$(CARGO) run -- manifest link \
		--source="$$MODEL_ID" \
		--target="$$DATASET_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL)
	@echo ""
	@echo "=== STEP 4: Validating the link ==="
	@MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	$(CARGO) run -- manifest validate \
		--id="$$MODEL_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL)
	@echo ""
	@echo "=== STEP 5: Showing the model manifest with its cross-references ==="
	@MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	$(CARGO) run -- manifest show \
		--id="$$MODEL_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL)
	@echo ""
	@echo "=== Cleaning up temporary files ==="
	@$(RM_CMD) dataset_output.txt model_output.txt

# Export provenance graph example
example-export-provenance: generate-keys setup-examples
	@echo "Step 1: Creating dataset..."
	$(CARGO) run -- dataset create \
		--paths=$(EXAMPLES_DATA_TRAIN_DIR)/data1.txt \
		--ingredient-names="Training Dataset" \
		--name="Training Dataset" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> dataset_output.txt
	
	@echo "Step 2: Creating model..."
	$(CARGO) run -- model create \
		--paths=$(EXAMPLES_MODELS_DIR)/model.onnx \
		--ingredient-names="ONNX Model" \
		--name="Trained Model" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> model_output.txt
	
	@echo "Step 3: Linking model to dataset..."
	@DATASET_ID=$$(grep -o "ID: [^ ]*" dataset_output.txt | cut -d' ' -f2); \
	MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	echo "Dataset ID: $$DATASET_ID"; \
	echo "Model ID: $$MODEL_ID"; \
	$(CARGO) run -- manifest link \
		--source="$$MODEL_ID" \
		--target="$$DATASET_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) > link_output.txt
	
	@echo "Step 4: Getting updated model ID..."
	@UPDATED_MODEL_ID=$$(grep -o "Updated manifest ID: [^ ]*" link_output.txt | cut -d' ' -f4 || echo "$$MODEL_ID"); \
	echo "Updated Model ID: $$UPDATED_MODEL_ID"
	
	@echo "Step 5: Exporting provenance graph..."
	@UPDATED_MODEL_ID=$$(grep -o "Updated manifest ID: [^ ]*" link_output.txt | cut -d' ' -f4 || echo "$$MODEL_ID"); \
	$(CARGO) run -- manifest export \
		--id="$$UPDATED_MODEL_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		--format=json \
		--output=provenance.json
	
	@echo "Provenance exported to provenance.json"
	@$(RM_CMD) dataset_output.txt model_output.txt link_output.txt

# Complex ML workflow example
example-complex-workflow: generate-keys setup-examples
	@echo "Creating dataset, model, and evaluation manifests..."
	
	@echo "Creating necessary files for complex workflow..."
	@echo "Initial training data" > $(EXAMPLES_DATA_TRAIN_DIR)/data1.txt
	@echo "Modified training data" > $(EXAMPLES_DATA_TRAIN_DIR)/data2.txt
	@$(MKDIR_CMD) $(EXAMPLES_DATA_DIR)/eval
	@echo "Evaluation data" > $(EXAMPLES_DATA_DIR)/eval/eval_data.txt
	@echo "Model weights" > $(EXAMPLES_MODELS_DIR)/model.onnx
	@echo '{"accuracy": 0.92, "f1": 0.89}' > $(EXAMPLES_RESULTS_DIR)/eval_results.json
	
	@echo "Step 1: Creating initial training dataset..."
	@$(CARGO) run -- dataset create \
		--paths=$(EXAMPLES_DATA_TRAIN_DIR)/data1.txt \
		--ingredient-names="Initial Training Data" \
		--name="Training Dataset V1" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> train_dataset_v1_output.txt
	
	@echo "Step 2: Creating modified training dataset..."
	@$(CARGO) run -- dataset create \
		--paths=$(EXAMPLES_DATA_TRAIN_DIR)/data2.txt \
		--ingredient-names="Modified Training Data" \
		--name="Training Dataset V2" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> train_dataset_v2_output.txt
	
	@echo "Step 3: Linking datasets (V1 → V2)..."
	@DATASET_V1_ID=$$(grep -o "ID: [^ ]*" train_dataset_v1_output.txt | cut -d' ' -f2); \
	DATASET_V2_ID=$$(grep -o "ID: [^ ]*" train_dataset_v2_output.txt | cut -d' ' -f2); \
	echo "Dataset V1 ID: $$DATASET_V1_ID"; \
	echo "Dataset V2 ID: $$DATASET_V2_ID"; \
	$(CARGO) run -- manifest link \
		--source="$$DATASET_V2_ID" \
		--target="$$DATASET_V1_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) > link_v1_v2_output.txt; \
	UPDATED_DATASET_V2_ID=$$(grep -o "Updated manifest ID: [^ ]*" link_v1_v2_output.txt | cut -d' ' -f4 || echo "$$DATASET_V2_ID"); \
	echo "Updated Dataset V2 ID: $$UPDATED_DATASET_V2_ID"; \
	echo "$$UPDATED_DATASET_V2_ID" > updated_dataset_v2_id.txt
	
	@echo "Step 4: Creating model..."
	@$(CARGO) run -- model create \
		--paths=$(EXAMPLES_MODELS_DIR)/model.onnx \
		--ingredient-names="Model Weights" \
		--name="Trained Model" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> model_output.txt

	@echo "Step 5: Linking model to training dataset V2..."
	@MODEL_ID=$$(grep -o "ID: [^ ]*" model_output.txt | cut -d' ' -f2); \
	UPDATED_DATASET_V2_ID=$$(cat updated_dataset_v2_id.txt); \
	echo "Model ID: $$MODEL_ID"; \
	echo "Training Dataset V2 ID: $$UPDATED_DATASET_V2_ID"; \
	$(CARGO) run -- manifest link \
		--source="$$MODEL_ID" \
		--target="$$UPDATED_DATASET_V2_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) > link_model_v2_output.txt; \
	UPDATED_MODEL_ID=$$(grep -o "Updated manifest ID: [^ ]*" link_model_v2_output.txt | cut -d' ' -f4 || echo "$$MODEL_ID"); \
	echo "Updated Model ID: $$UPDATED_MODEL_ID"; \
	echo "$$UPDATED_MODEL_ID" > updated_model_id.txt
	
	@echo "Step 6: Creating evaluation dataset..."
	@$(CARGO) run -- dataset create \
		--paths=$(EXAMPLES_DATA_DIR)/eval/eval_data.txt \
		--ingredient-names="Evaluation Data" \
		--name="Evaluation Dataset" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> eval_dataset_output.txt
	
	@echo "Step 7: Creating evaluation results linked to model and evaluation dataset..."
	@UPDATED_MODEL_ID=$$(cat updated_model_id.txt); \
	EVAL_DATASET_ID=$$(grep -o "ID: [^ ]*" eval_dataset_output.txt | cut -d' ' -f2); \
	echo "Updated Model ID: $$UPDATED_MODEL_ID"; \
	echo "Evaluation Dataset ID: $$EVAL_DATASET_ID"; \
	$(CARGO) run -- evaluation create \
		--path=$(EXAMPLES_RESULTS_DIR)/eval_results.json \
		--name="Model Evaluation" \
		--model-id="$$UPDATED_MODEL_ID" \
		--dataset-id="$$EVAL_DATASET_ID" \
		--metrics=accuracy=0.92,f1=0.89 \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		> eval_output.txt
	
	@echo "Step 8: Exporting complete provenance graph with full depth..."
	@EVAL_ID=$$(grep -o "ID: [^ ]*" eval_output.txt | cut -d' ' -f2); \
	echo "Evaluation ID: $$EVAL_ID"; \
	$(CARGO) run -- manifest export \
		--id="$$EVAL_ID" \
		--storage-type=database \
		--storage-url=$(DEFAULT_STORAGE_URL) \
		--format=json \
		--max-depth=10 \
		--output=full_provenance.json
	
	@echo "Cleaning up temporary output files..."
	@$(RM_CMD) train_dataset_v1_output.txt train_dataset_v2_output.txt model_output.txt \
		eval_dataset_output.txt eval_output.txt link_v1_v2_output.txt \
		link_model_v2_output.txt updated_dataset_v2_id.txt updated_model_id.txt
	
	@echo "Workflow complete! Full provenance exported to full_provenance.json"

# Run all examples
examples: example-model-single example-model-multi example-dataset-single example-dataset-multi \
          example-full-workflow example-model-dataset-workflow example-export-provenance \
          example-complex-workflow example-filesystem-storage

# Filesystem storage example
example-filesystem-storage: generate-keys setup-examples
	@echo "Creating filesystem storage directory..."
	@$(MKDIR_CMD) $(DEFAULT_FILESYSTEM_PATH)
	@echo "Creating and storing model manifest in filesystem..."
	$(CARGO) run -- model create \
		--paths=$(EXAMPLES_MODELS_DIR)/model.onnx \
		--ingredient-names="ONNX Model" \
		--name="Filesystem Storage Model" \
		--author-org="Test Org" \
		--author-name="Test Author" \
		--storage-type=local-fs \
		--storage-url=$(DEFAULT_FILESYSTEM_PATH) \
		--format=json
	@echo "Listing manifests from filesystem storage..."
	$(CARGO) run -- model list \
		--storage-type=local-fs \
		--storage-url=$(DEFAULT_FILESYSTEM_PATH)

# Help text for example targets
help-examples:
	@echo "Example targets for atlas-cli:"
	@echo "=================================================================================="
	@echo "Basic examples:"
	@echo "  make example-model-single   - Run single model example"
	@echo "  make example-model-multi    - Run multi-model example"
	@echo "  make example-dataset-single - Run single dataset example"
	@echo "  make example-dataset-multi  - Run multi-dataset example"
	@echo ""
	@echo "Workflow examples:"
	@echo "  make example-model-dataset-workflow - Run example that links model to dataset"
	@echo "  make example-full-workflow - Run full workflow example with step-by-step guidance"
	@echo "  make example-export-provenance - Export provenance graph example"
	@echo "  make example-complex-workflow - Run complex workflow with evaluation"
	@echo ""
	@echo "Storage examples:"
	@echo "  make example-filesystem-storage - Example using filesystem storage backend"
	@echo ""
	@echo "Utility targets:"
	@echo "  make setup-examples - Create example directory structure"
	@echo "  make examples      - Run all examples"
	@echo "  make run-json      - Run example with JSON output"
	@echo "  make run-cbor      - Run example with CBOR output"