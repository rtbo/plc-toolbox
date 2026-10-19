#include "common.h"
#include "open62541/types.h"
#include <open62541/client.h>
#include <open62541/client_config_default.h>
#include <open62541/client_highlevel.h>
#include <open62541/plugin/log_stdout.h>
#include <stdio.h>

int main(void) {
  const char *dirname = getenv("CERTS_DIR");
  if (!dirname) {
    fprintf(stderr, "CERTS_DIR environment variable is not set.\n");
    return EXIT_FAILURE;
  }

  char certPath[256] = {0};
  char keyPath[256] = {0};
  snprintf(certPath, sizeof(certPath), "%s/client-cert.der", dirname);
  snprintf(keyPath, sizeof(keyPath), "%s/client-key.der", dirname);

  // 1. Charger le certificat et la clé privée du client
  UA_ByteString certificate = loadFile(certPath);
  UA_ByteString privateKey = loadFile(keyPath);
  // 2. Charger le certificat du serveur pour établir la confiance (requis pour
  // chiffrer)
  // UA_ByteString serverCertificate = loadFile("server_cert.der");

  UA_Client *client = UA_Client_new();
  UA_ClientConfig *config = UA_Client_getConfig(client);

  // 3. Configurer la sécurité du client (Basic256Sha256)
  UA_StatusCode retval = UA_ClientConfig_setDefaultEncryption(
      config, certificate, privateKey, NULL, 0, NULL, 0);

  if (retval != UA_STATUSCODE_GOOD) {
    printf("Erreur de configuration de l'encryption client\n");
    UA_ByteString_clear(&certificate);
    UA_ByteString_clear(&privateKey);
    // UA_ByteString_clear(&serverCertificate);
    UA_Client_delete(client);
    return 1;
  }

  config->clientDescription.applicationUri =
      UA_STRING_ALLOC("urn:plc-toolbox:ua-lab:client");

  // // 4. Forcer explicitement le mode de sécurité et l'URI de la politique si
  // // nécessaire (Par défaut, UA_ClientConfig_setDefaultEncryption configure
  // // Basic256Sha256)
  // config->securityMode = UA_MESSAGESECURITYMODE_SIGNANDENCRYPT;
  // UA_String_clear(&config->securityPolicyUri);
  // config->securityPolicyUri = UA_STRING_ALLOC(
  //     "http://opcfoundation.org/UA/SecurityPolicy#Basic256Sha256");

  // 5. Connexion au serveur via la méthode standard UA_Client_connect
  retval = UA_Client_connect(client, "opc.tcp://127.0.0.1:4840");

  if (retval != UA_STATUSCODE_GOOD) {
    printf("Échec de la connexion sécurisée au serveur (Code: 0x%x)\n", retval);
    UA_Client_delete(client);
    return 1;
  }

  printf("Client connecté au serveur OPC/UA de manière sécurisée "
         "(SignAndEncrypt) !\n");

  // Exemple de lecture de l'heure courante du serveur
  UA_Variant value;
  UA_Variant_init(&value);
  retval = UA_Client_readValueAttribute(
      client, UA_NODEID_NUMERIC(0, UA_NS0ID_SERVER_SERVERSTATUS_CURRENTTIME),
      &value);

  if (retval == UA_STATUSCODE_GOOD &&
      UA_Variant_hasScalarType(&value, &UA_TYPES[UA_TYPES_DATETIME])) {
    UA_DateTime dateTime = *(UA_DateTime *)value.data;
    printf("Heure courante lue avec succès sur le serveur sécurisé !\n");
  }

  UA_Variant_clear(&value);

  // Déconnexion et nettoyage
  UA_Client_disconnect(client);
  UA_Client_delete(client);

  // Nettoyage des buffers de certificats alloués
  UA_ByteString_clear(&certificate);
  UA_ByteString_clear(&privateKey);
  //UA_ByteString_clear(&serverCertificate);

  return 0;
}
