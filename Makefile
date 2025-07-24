# Minimal Makefile for TicTac Online
# Detects Docker or Podman automatically

# Container runtime detection
CONTAINER_CMD := $(shell command -v podman 2> /dev/null || command -v docker 2> /dev/null)

.PHONY: build deploy clean status urls open-client tunnel forward forward-client forward-server log-server log-client stop-server stop-client start-client start-server stop start

# Show status
status:
	kubectl get all -n tictac

# Show URLs
urls:
	@if command -v minikube > /dev/null && minikube status > /dev/null 2>&1; then \
		if [[ "$$(uname)" == "Darwin" ]]; then \
			echo "On macOS with Minikube + Podman, use one of these methods:"; \
			echo ""; \
			echo "Method 1 (Recommended): Run 'make forward' in another terminal, then use:"; \
			echo "  Client: http://localhost:3000"; \
			echo "  API: http://localhost:8080"; \
			echo ""; \
			echo "Method 2: Run 'make tunnel' (minikube tunnel) in another terminal, then use:"; \
			echo "  Client: http://localhost:30030"; \
			echo "  API: http://localhost:30080"; \
			echo ""; \
			echo "Method 3: Run 'make open-client' to let Minikube handle it"; \
		else \
			echo "Client: http://$$(minikube ip):30030"; \
			echo "API: http://$$(minikube ip):30080"; \
		fi \
	else \
		echo "Client: http://localhost:30030"; \
		echo "API: http://localhost:30080"; \
	fi

# Open client in browser (works with Minikube)
open-client:
	@if command -v minikube > /dev/null && minikube status > /dev/null 2>&1; then \
		echo "Opening client in browser..."; \
		minikube service client -n tictac; \
	else \
		echo "Opening http://localhost:30030"; \
		open http://localhost:30030 || xdg-open http://localhost:30030 || echo "Please open http://localhost:30030 in your browser"; \
	fi

# Start minikube tunnel (for macOS)
tunnel:
	@echo "Starting minikube tunnel (keep this running)..."
	@echo "Press Ctrl+C to stop"
	minikube tunnel

# Port forward client (alternative to tunnel)
forward-client:
	@echo "Starting port forward for client (http://localhost:3000)..."
	@echo "Press Ctrl+C to stop"
	kubectl port-forward -n tictac service/client 3000:3000

# Port forward server (alternative to tunnel)
forward-server:
	@echo "Starting port forward for server (http://localhost:8080)..."
	@echo "Press Ctrl+C to stop"
	kubectl port-forward -n tictac service/server 8080:8080

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
	@make urls
	@echo ""
	@echo "Quick commands:"
	@echo "  make open-client  - Open client in browser"
	@echo "  make tunnel       - Start tunnel for localhost access (macOS)"
	@echo "  make urls         - Show access URLs"

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

# Get server logs (with optional line count)
log-%:
	kubectl logs deployment/$* -n tictac --tail=$(or $(LINES),50) -f

# Scale server to 0 (stop)
stop-%:
	kubectl scale deployment/$* -n tictac --replicas=0
	@echo "$* stopped"

# Scale server to 1 (start)
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
