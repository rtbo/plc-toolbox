#include <stdlib.h>

#include <open62541/plugin/certificategroup_default.h>
#include <open62541/plugin/log.h>
#include <open62541/plugin/log_stdout.h>
#include <open62541/plugin/securitypolicy.h>
#include <open62541/server.h>
#include <open62541/server_config_default.h>

#include "common.h"

int main(void) {
  const char *dirname = getenv("CERTS_DIR");
  if (!dirname) {
    fprintf(stderr, "CERTS_DIR environment variable is not set.\n");
    return EXIT_FAILURE;
  }

  char certPath[256] = {0};
  char keyPath[256] = {0};
  snprintf(certPath, sizeof(certPath), "%s/server-cert.der", dirname);
  snprintf(keyPath, sizeof(keyPath), "%s/server-key.der", dirname);

  // 1. Load the server certificate and private key
  UA_ByteString certificate = loadFile(certPath);
  UA_ByteString privateKey = loadFile(keyPath);
  
  UA_Server *server = UA_Server_new();
  UA_ServerConfig *config = UA_Server_getConfig(server);

  /* Exchange the logger */
  UA_Logger logger = UA_Log_Stdout_withLevel(UA_LOGLEVEL_INFO);
  logger.clear = config->logging->clear;
  *config->logging = logger;

  // 2. Configure the security layer with the loaded certificates
  // Use of Basic256Sha256 with a default certificate storage (AcceptAll for
  // testing)
  UA_StatusCode retval = UA_ServerConfig_setDefaultWithSecurityPolicies(
      config, 4840, &certificate, &privateKey, NULL, 0, NULL, 0, NULL, 0);
  
    config->applicationDescription.applicationUri =
        UA_STRING_ALLOC("urn:plc-toolbox:ua-lab:server");

  if (retval != UA_STATUSCODE_GOOD) {
    UA_LOG_INFO(&logger, UA_LOGCATEGORY_SERVER,
                "Erreur lors de l'activation de la securite !");
    UA_ByteString_clear(&certificate);
    UA_ByteString_clear(&privateKey);
    UA_Server_delete(server);
    return (int)retval;
  }

  // Démarrage du serveur
  UA_Boolean running = true;
  retval = UA_Server_run(server, &running);

  UA_ByteString_clear(&certificate);
  UA_ByteString_clear(&privateKey);

  UA_Server_delete(server);
  return (int)retval;
}
