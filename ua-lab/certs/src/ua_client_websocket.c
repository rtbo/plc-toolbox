#include <open62541/client_config_default.h>
#include <open62541/client_highlevel.h>
#include <open62541/plugin/log_stdout.h>

#include <stdio.h>
#include <stdlib.h>

int
main(int argc, char **argv) {
    const char *url = argc > 1 ? argv[1] : "opc.ws://localhost:4843/opcua";

    UA_Client *client = UA_Client_new();
    UA_ClientConfig *cc = UA_Client_getConfig(client);
    UA_ClientConfig_setDefault(cc);

    /* Trust the self-signed server certificate (PEM or DER) */
    if(argc > 2) {
        FILE *f = fopen(argv[2], "rb");
        if(f) {
            UA_ByteString_allocBuffer(&cc->webSocketCaCertificate, 65536);
            cc->webSocketCaCertificate.length =
                fread(cc->webSocketCaCertificate.data, 1, 65536, f);
            fclose(f);
        }
    }

    UA_StatusCode res = UA_Client_connect(client, url);
    if(res != UA_STATUSCODE_GOOD) {
        UA_LOG_ERROR(UA_Log_Stdout, UA_LOGCATEGORY_USERLAND,
                     "Connect to %s failed: %s", url, UA_StatusCode_name(res));
        UA_Client_delete(client);
        return EXIT_FAILURE;
    }

    UA_Variant value;
    UA_Variant_init(&value);
    res = UA_Client_readValueAttribute(
        client, UA_NODEID_NUMERIC(0, UA_NS0ID_SERVER_SERVERSTATUS_CURRENTTIME), &value);
    if(res == UA_STATUSCODE_GOOD && UA_Variant_hasScalarType(&value, &UA_TYPES[UA_TYPES_DATETIME])) {
        UA_DateTimeStruct dts = UA_DateTime_toStruct(*(UA_DateTime *)value.data);
        UA_LOG_INFO(UA_Log_Stdout, UA_LOGCATEGORY_USERLAND,
                    "Server time: %02u:%02u:%02u", dts.hour, dts.min, dts.sec);
    }
    UA_Variant_clear(&value);

    UA_Client_delete(client);
    return EXIT_SUCCESS;
}
