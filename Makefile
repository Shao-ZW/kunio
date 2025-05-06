IMAGE_NAME = iouring-dev
CONTAINER_NAME = iouring-container

image:
	@if ! docker images | grep -q $(IMAGE_NAME); then \
        docker build -t $(IMAGE_NAME) .; \
		echo "Image $(IMAGE_NAME) create successfully"; \
	else \
        echo "Image $(IMAGE_NAME) already exists."; \
    fi

container:
	@if ! docker ps -a | grep -q $(CONTAINER_NAME); then \
		docker create -it --name $(CONTAINER_NAME) -v $(shell pwd):/app $(IMAGE_NAME); \
		echo "Container $(CONTAINER_NAME) created successfully"; \
	else \
		echo "Container $(CONTAINER_NAME) already exists."; \
	fi	

run:
	docker start $(CONTAINER_NAME) 
	docker exec -it $(CONTAINER_NAME) bash

clean:
	docker rm -f $(CONTAINER_NAME) >/dev/null 2>&1 || true
	@echo "Cleaned up container."

clean_all:
	docker rm -f $(CONTAINER_NAME) >/dev/null 2>&1 || true
	docker rmi -f $(IMAGE_NAME) >/dev/null 2>&1 || true
	@echo "Cleaned up container and image."
