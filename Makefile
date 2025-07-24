# Minimal Makefile for TicTac Online
# Detects Docker or Podman automatically

# Container runtime detection
CONTAINER_CMD := $(shell command -v podman 2> /dev/null || command -v docker 2> /dev/null)

.PHONY: build deploy clean status forward forward-client forward-server forward-db log-server log-client stop-server stop-client start-client start-server stop start

# Show status
status:
	kubectl get all -n tictac

# Port forward both services
forward:
	@echo "Starting port forwards..."
	@echo "Client will be available at: http://localhost:3000"
	@echo "Server will be available at: http://localhost:8080"
	@echo "Press Ctrl+C to stop"
	@trap 'kill %1 %2' INT; \
	kubectl port-forward -n tictac service/client 3000:3000 & \
	kubectl port-forward -n tictac service/server 8080:8080 & \
	wait

# Port forward database
forward-db:
	@echo "Starting database port forward..."
	@echo "SurrealDB will be available at: http://localhost:8000"
	@echo "Press Ctrl+C to stop"
	kubectl port-forward -n tictac service/surrealdb 8000:8000

# Build individual images
build-%:
	@echo "Building $* with $(CONTAINER_CMD)..."
	@cd $* && $(CONTAINER_CMD) build -t localhost/tictac-$*:latest .

# Build both images
build: build-server build-client
	@echo "Build complete!"

# Deploy to Kubernetes
deploy: build
	@echo "Deploying to Kubernetes..."
	kubectl apply -f k8s-minimal.yaml
	@echo "Waiting for deployments..."
	kubectl wait --for=condition=available --timeout=300s deployment --all -n tictac
	@echo "Deployment complete!"
	@echo ""
	@echo "To access the application, run:"
	@echo "  make forward"
	@echo ""
	@echo "Then open:"
	@echo "  Client: http://localhost:3000"
	@echo "  API: http://localhost:8080"

# Clean up everything
clean:
	kubectl delete namespace tictac --ignore-not-found=true
	@echo "Cleaned up Kubernetes resources"

# Restart a specific service
restart-%:
	kubectl rollout restart deployment/$* -n tictac
	kubectl rollout status deployment/$* -n tictac

# Quick development cycle - rebuild and restart
dev-%: build-%
	kubectl rollout restart deployment/$* -n tictac
	kubectl rollout status deployment/$* -n tictac

# Get logs (with optional line count)
log-%:
	kubectl logs deployment/$* -n tictac --tail=$(or $(LINES),50) -f

# Scale to 0 (stop)
stop-%:
	kubectl scale deployment/$* -n tictac --replicas=0
	@echo "$* stopped"

# Scale to 1 (start)
start-%:
	kubectl scale deployment/$* -n tictac --replicas=1
	kubectl wait --for=condition=available --timeout=60s deployment/$* -n tictac
	@echo "$* started"

# Stop all services (except database)
stop: stop-server stop-client
	@echo "Server and client stopped"

# Start all services
start: start-server start-client
	@echo "Server and client started"
