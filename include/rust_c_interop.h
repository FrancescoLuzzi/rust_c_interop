#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct client_t client_t;

struct client_t *init_connection(const char *host,
                                 unsigned int port,
                                 const char *username,
                                 const char *password,
                                 const char *database);

bool simple_query(struct client_t *client, const char *table);

void free_client(struct client_t *client);
