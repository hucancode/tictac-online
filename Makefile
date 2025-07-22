.PHONY: help build-server build-client deploy clean minikube-start minikube-stop status logs-server logs-client logs-db dashboard start-port-forward dev stop-server stop-client start-server start-client reset

# Variables
MINIKUBE_PROFILE ?= tictac
SERVER_IMAGE = tictac-server:latest
CLIENT_IMAGE = tictac-client:latest


# Colors
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
NC := \033[0m # No Color

help: ## Show this help
	@echo "Container Runtime: $(RUNTIME_NAME)"
	@echo "Minikube Profile: $(MINIKUBE_PROFILE)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Quick start: make dev"

minikube-start: ## Start Minikube cluster
	@if [ "$(RUNTIME_NAME)" = "podman" ]; then \
		echo "$(YELLOW)Starting Minikube with Podman driver...$(NC)"; \
		minikube start --profile=$(MINIKUBE_PROFILE) --driver=podman --container-runtime=cri-o --memory=3072 --cpus=2; \
	else \
		echo "$(YELLOW)Starting Minikube with Docker driver...$(NC)"; \
		minikube start --profile=$(MINIKUBE_PROFILE) --driver=docker --memory=3072 --cpus=2; \
	fi
	minikube profile $(MINIKUBE_PROFILE)
	@if [ "$(RUNTIME_NAME)" = "docker" ]; then \
		echo "Minikube started. Run 'eval $$(minikube docker-env)' to use Minikube's Docker daemon"; \
	else \
		echo "Minikube started with Podman driver. Images will be loaded with 'minikube image load'"; \
	fi

minikube-stop: ## Stop Minikube cluster
	minikube stop --profile=$(MINIKUBE_PROFILE)

# Detect container runtime
CONTAINER_RUNTIME := $(shell command -v docker 2> /dev/null || command -v podman 2> /dev/null)
ifeq ($(CONTAINER_RUNTIME),)
    $(error Neither Docker nor Podman is installed)
endif
RUNTIME_NAME := $(notdir $(CONTAINER_RUNTIME))

build-server: ## Build server container image
	@echo "Building server with $(RUNTIME_NAME)..."
	cd server && $(CONTAINER_RUNTIME) build -t localhost/$(SERVER_IMAGE) .
	@if [ "$(RUNTIME_NAME)" = "podman" ]; then \
		podman save localhost/$(SERVER_IMAGE) | minikube image load --profile=$(MINIKUBE_PROFILE) -; \
	fi

build-client: ## Build client container image
	@echo "Building client with $(RUNTIME_NAME)..."
	cd client && $(CONTAINER_RUNTIME) build -t localhost/$(CLIENT_IMAGE) .
	@if [ "$(RUNTIME_NAME)" = "podman" ]; then \
		podman save localhost/$(CLIENT_IMAGE) | minikube image load --profile=$(MINIKUBE_PROFILE) -; \
	fi

deploy: ## Deploy to Minikube
	kubectl apply -f tictac-k8s.yaml
	@echo "$(YELLOW)Waiting for deployments to be ready...$(NC)"
	@kubectl wait --namespace=tictac --for=condition=available --timeout=300s deployment/surrealdb || exit 1
	@kubectl wait --namespace=tictac --for=condition=available --timeout=300s deployment/server || exit 1
	@kubectl wait --namespace=tictac --for=condition=available --timeout=300s deployment/client || exit 1
	@echo "$(GREEN)✅ All deployments are ready!$(NC)"

clean: ## Remove all Kubernetes resources
	kubectl delete -f tictac-k8s.yaml || true

status: ## Check deployment status
	kubectl get all -n tictac

logs-server: ## View server logs
	kubectl logs -n tictac -l app=server -f

logs-client: ## View client logs
	kubectl logs -n tictac -l app=client -f

logs-db: ## View SurrealDB logs
	kubectl logs -n tictac -l app=surrealdb -f

dashboard: ## Open Kubernetes dashboard
	minikube dashboard --profile=$(MINIKUBE_PROFILE)

start-port-forward: ## Start port forwarding in background
	@echo "$(YELLOW)Setting up port forwarding...$(NC)"
	@pkill -f "kubectl port-forward.*tictac" || true
	@kubectl port-forward -n tictac service/client 30030:3000 > /dev/null 2>&1 &
	@kubectl port-forward -n tictac service/server 30080:8080 > /dev/null 2>&1 &
	@kubectl port-forward -n tictac service/surrealdb 8000:8000 > /dev/null 2>&1 &
	@sleep 2
	@echo "$(GREEN)✅ Port forwarding started$(NC)"

# Main deployment command
dev: build-server build-client deploy start-port-forward ## Complete deployment (main command)
	@echo ""
	@echo "$(GREEN)✅ Deployment complete!$(NC)"
	@echo ""
	@echo "$(GREEN)Application is running at:$(NC)"
	@echo "  Client:    $(YELLOW)http://localhost:30030$(NC)"
	@echo "  API:       $(YELLOW)http://localhost:30080$(NC)"
	@echo "  Database:  $(YELLOW)localhost:8000$(NC)"
	@echo ""
	@echo "$(GREEN)Useful commands:$(NC)"
	@echo "  make logs-server    # View server logs"
	@echo "  make logs-client    # View client logs"
	@echo "  make status         # Check deployment status"
	@echo "  make dashboard      # Open Kubernetes dashboard"

# Development helpers
stop-server: ## Stop server deployment (for local server development)
	@echo "$(YELLOW)Scaling down server deployment...$(NC)"
	kubectl scale deployment/server -n tictac --replicas=0
	@echo "$(GREEN)✓ Server stopped. You can now run 'cd server && cargo run'$(NC)"
	@echo "$(YELLOW)Note: SurrealDB is already available at localhost:8000$(NC)"

stop-client: ## Stop client deployment (for local client development)
	@echo "$(YELLOW)Scaling down client deployment...$(NC)"
	kubectl scale deployment/client -n tictac --replicas=0
	@echo "$(GREEN)✓ Client stopped. You can now run 'cd client && bun run dev'$(NC)"
	@echo "$(YELLOW)Note: Client will connect to API at localhost:30080$(NC)"

start-server: ## Restart server deployment
	@echo "$(YELLOW)Scaling up server deployment...$(NC)"
	kubectl scale deployment/server -n tictac --replicas=1
	@kubectl wait --namespace=tictac --for=condition=available --timeout=60s deployment/server
	@echo "$(GREEN)✓ Server restarted$(NC)"

start-client: ## Restart client deployment
	@echo "$(YELLOW)Scaling up client deployment...$(NC)"
	kubectl scale deployment/client -n tictac --replicas=1
	@kubectl wait --namespace=tictac --for=condition=available --timeout=60s deployment/client
	@echo "$(GREEN)✓ Client restarted$(NC)"
