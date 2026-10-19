#include <stdio.h>
#include <open62541/types.h>

// Utility function to load a file into a UA_ByteString
static UA_ByteString loadFile(const char *path) {
    FILE *f = fopen(path, "rb");
    if(!f) return UA_BYTESTRING_NULL;
    
    fseek(f, 0, SEEK_END);
    size_t size = ftell(f);
    fseek(f, 0, SEEK_SET);
    
    UA_ByteString buf;
    if(UA_ByteString_allocBuffer(&buf, size) != UA_STATUSCODE_GOOD) {
        fclose(f);
        return UA_BYTESTRING_NULL;
    }
    
    fread(buf.data, 1, size, f);
    fclose(f);
    return buf;
}
