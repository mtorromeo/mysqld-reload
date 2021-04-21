.PHONY: all
all: server-system-variables.html replication-options-binary-log.html

server-system-variables.html:
	wget https://dev.mysql.com/doc/refman/8.0/en/server-system-variables.html -O $@

replication-options-binary-log.html:
	wget https://dev.mysql.com/doc/refman/8.0/en/replication-options-binary-log.html -O $@
