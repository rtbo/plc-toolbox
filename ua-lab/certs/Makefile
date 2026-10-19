ifndef OPEN62541_PREFIX
    $(error OPEN62541_PREFIX is not set. Please set it to the installation path of open62541.)
endif

CFLAGS = -I$(OPEN62541_PREFIX)/include -Wall -O2
LDFLAGS = -L$(OPEN62541_PREFIX)/lib64 -lopen62541 -lcrypto

# Compilateur
CC = gcc

# Cibles
all: bin/server certs/server-cert.pem certs/server-cert.der certs/server-key.pem certs/server-key.der \
	bin/client certs/client-cert.pem certs/client-cert.der certs/client-key.pem certs/client-key.der

certs/server-key.pem: 
	openssl genrsa -out certs/server-key.pem 2048
	
certs/server-cert.pem: certs/server-key.pem src/server.cnf
	openssl req -new -x509 -days 365 -key certs/server-key.pem -out certs/server-cert.pem -config src/server.cnf -extensions v3_req

certs/server-cert.der: certs/server-cert.pem
	openssl x509 -in certs/server-cert.pem -outform DER -out certs/server-cert.der

certs/server-key.der: certs/server-key.pem
	openssl rsa -in certs/server-key.pem -outform DER -out certs/server-key.der

bin/server: src/server.c 
	$(CC) src/server.c -o bin/server $(CFLAGS) $(LDFLAGS)


certs/client-key.pem: 
	openssl genrsa -out certs/client-key.pem 2048
	
certs/client-cert.pem: certs/client-key.pem src/client.cnf
	openssl req -new -x509 -days 365 -key certs/client-key.pem -out certs/client-cert.pem -config src/client.cnf -extensions v3_req

certs/client-cert.der: certs/client-cert.pem
	openssl x509 -in certs/client-cert.pem -outform DER -out certs/client-cert.der

certs/client-key.der: certs/client-key.pem
	openssl rsa -in certs/client-key.pem -outform DER -out certs/client-key.der
	
bin/client: src/client.c
	$(CC) src/client.c -o bin/client $(CFLAGS) $(LDFLAGS)

# Nettoyage des exécutables
clean:
	rm -f bin/server bin/client certs/server-cert.pem certs/server-cert.der certs/server-key.pem certs/server-key.der certs/client-cert.pem certs/client-cert.der certs/client-key.pem certs/client-key.der

# Nettoyage complet (incluant les certificats si besoin)
clean-all: clean

.PHONY: all clean clean-all