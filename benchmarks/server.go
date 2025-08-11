package main

import (
    "fmt"
    "net/http"
)

func handler(w http.ResponseWriter, r *http.Request) {
    w.WriteHeader(http.StatusOK)
    w.Write([]byte("Hello, World!"))
}

func main() {
    http.HandleFunc("/", handler)
    fmt.Println("Go server listening on :8000")
    if err := http.ListenAndServe(":8000", nil); err != nil {
        panic(err)
    }
}

