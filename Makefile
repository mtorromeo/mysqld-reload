.PHONY: all
all: src/dynamic-system-variables.txt

dynamic-system-variables.html:
	wget https://dev.mysql.com/doc/refman/8.0/en/dynamic-system-variables.html -O $@

src/dynamic-system-variables.txt: dynamic-system-variables.html
	echo '[' > $@
	grep '</tr><tr><th scope="row"><a class="link" href=' dynamic-system-variables.html | sed -r 's@</tr><tr><th scope="row"><a class="link" href=.*>(.*)</a></th>@"\1",@' >> $@
	echo ']' >> $@
