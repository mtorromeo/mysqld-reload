.PHONY: all
all: server-system-variables.html

server-system-variables.html:
	wget https://dev.mysql.com/doc/refman/8.0/en/server-system-variables.html -O $@
