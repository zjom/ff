# Http

Blocking HTTP client. Import with:

```ff
Http = import "Http"
```

Each call returns `[:ok, response]` or `[:error, msg]`. A response is
an object:

```ff
{
  :status: 200,
  :headers: {"content-type": "application/json", ..},
  :body: "..."
}
```

4xx/5xx responses come back as `[:ok, resp]` with the status set —
they are not converted into errors, since the caller usually wants to
inspect the response either way.

## Functions

| Function                       | Description                       |
|--------------------------------|-----------------------------------|
| `Http.get(url)`                | GET                                |
| `Http.head(url)`               | HEAD (no body)                     |
| `Http.delete(url)`             | DELETE                             |
| `Http.post(url, body)`         | POST with string body              |
| `Http.put(url, body)`          | PUT with string body               |
| `Http.patch(url, body)`        | PATCH with string body             |
| `Http.request(opts)`           | Arbitrary request, see below       |

### `Http.request(opts)`

`opts` is an object:

```ff
{
  :url:     "https://example.com/api",
  :method:  "POST",                          # optional, defaults to "GET"
  :headers: {"content-type": "application/json"},  # optional
  :body:    "{\"k\":1}"                       # optional
}
```

```ff
match Http.request({:url: "https://example.com", :method: "GET"})
  [:ok, r]      -> println("status: " + r.status),
  [:error, msg] -> println("failed: " + msg)
```
