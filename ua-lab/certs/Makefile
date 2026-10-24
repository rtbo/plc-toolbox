ifndef OPEN62541_PREFIX
$(error OPEN62541_PREFIX is not set. Please set it to the installation path of open62541.)
endif

ifndef CC
$(info CC is not set. Using default gcc)
CC = gcc
endif

CFLAGS = -I$(OPEN62541_PREFIX)/include -Wall -O2
LDFLAGS = -L$(OPEN62541_PREFIX)/lib64 -lopen62541 -lcrypto -lssl -lwebsockets -lpthread

# Cibles
all: certs/server-cert.pem certs/server-cert.der certs/server-key.pem certs/server-key.der \
	certs/client-cert.pem certs/client-cert.der certs/client-key.pem certs/client-key.der \
	bin/server bin/client \
	bin/server_websocket bin/client_websocket

	
certs/server-key.pem: 
	openssl genrsa -out certs/server-key.pem 2048
	
certs/server-cert.pem: certs/server-key.pem src/server.cnf
	openssl req -new -x509 -days 365 -key certs/server-key.pem -out certs/server-cert.pem -config src/server.cnf -extensions v3_req

certs/server-cert.der: certs/server-cert.pem
	openssl x509 -in certs/server-cert.pem -outform DER -out certs/server-cert.der

certs/server-key.der: certs/server-key.pem
	openssl rsa -in certs/server-key.pem -outform DER -out certs/server-key.der


certs/client-key.pem: 
	openssl genrsa -out certs/client-key.pem 2048
	
certs/client-cert.pem: certs/client-key.pem src/client.cnf
	openssl req -new -x509 -days 365 -key certs/client-key.pem -out certs/client-cert.pem -config src/client.cnf -extensions v3_req

certs/client-cert.der: certs/client-cert.pem
	openssl x509 -in certs/client-cert.pem -outform DER -out certs/client-cert.der

certs/client-key.der: certs/client-key.pem
	openssl rsa -in certs/client-key.pem -outform DER -out certs/client-key.der
	

bin/server: src/server.c 
	$(CC) src/server.c -o bin/server $(CFLAGS) $(LDFLAGS)
	
bin/client: src/client.c
	$(CC) src/client.c -o bin/client $(CFLAGS) $(LDFLAGS)

bin/server_websocket: src/server_websocket.c
	$(CC) src/server_websocket.c -o bin/server_websocket $(CFLAGS) $(LDFLAGS)

bin/client_websocket: src/client_websocket.c
	$(CC) src/client_websocket.c -o bin/client_websocket $(CFLAGS) $(LDFLAGS)

# Nettoyage des exécutables
clean:
	rm -f certs/server-cert.pem certs/server-cert.der certs/server-key.pem certs/server-key.der \
	    certs/client-cert.pem certs/client-cert.der certs/client-key.pem certs/client-key.der \
		bin/server bin/client bin/server_websocket bin/client_websocket
					

# Nettoyage complet (incluant les certificats si besoin)
clean-all: clean

.PHONY: all clean clean-all