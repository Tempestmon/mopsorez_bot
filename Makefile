TENANT = tempestmon
BOT_VERSION = 0.0.1
PROJECT = mopsorez_bot

bot_build:
	docker build -t $(TENANT)/$(PROJECT):$(BOT_VERSION) -f docker/Dockerfile .

bot_tag:
	docker tag $(TENANT)/$(PROJECT):$(BOT_VERSION) $(TENANT)/bot:latest

bot_push: 
	docker push $(TENANT)/$(PROJECT):$(BOT_VERSION) && docker push $(TENANT)/$(PROJECT):latest

bot_run:
	docker run --rm -it (TENANT)/$(PROJECT):$(BOT_VERSION)
