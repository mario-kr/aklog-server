#!/bin/bash

TEST_CONFIG="./test_config.toml"
TEST_HOST="localhost"
TEST_PORT="8080"

TEST_GLOBAL_RESULT=0

cd "$(dirname "${BASH_SOURCE[0]}")"

cargo run -- \
    --config "$TEST_CONFIG" \
    --address "$TEST_HOST" \
    --port "$TEST_PORT" \
    >/dev/null 2>&1 &
TEST_PID="$!"
sleep 1

if ! [ -e /proc/"$TEST_PID" ]; then
    echo "Failed to run aklog-server (debug build). Aborting."
    exit 1
fi

test_index_endpoint() {
    curl -XGET -v "${TEST_HOST}:${TEST_PORT}" 2>&1 | grep -q '< HTTP/1.1 200 OK'
    if [ $? == 0 ]; then
        echo "Success: test_index_endpoint"
    else
        echo "Failure: test_index_endpoint"
        TEST_GLOBAL_RESULT=1
    fi
}


test_search_endpoint() {
    search_result=$(curl \
        --silent \
        -H 'Content-Type: application/json' \
        --data '{"target":""}' \
        -XPOST \
        "${TEST_HOST}:${TEST_PORT}/search")
    if [ "$search_result" == '["test_data.data"]' ]; then
        echo "Success: test_search_endpoint"
    else
        echo "Failure: test_search_endpoint"
        TEST_GLOBAL_RESULT=1
    fi
}

test_annotations_endpoint() {
    echo "Warning: test_annotations_endpoint: Feature not implemented in aklog-server"
}

test_query_endpoint() {
    QUERY_JSON="./query_request.json"
    query_result=$(curl \
        --silent \
        -H 'Content-Type: application/json' \
        --data @"$QUERY_JSON" \
        -XPOST \
        "${TEST_HOST}:${TEST_PORT}/query")
    if [ "$query_result" == '[{"target":"test_data.data","datapoints":[[2.0,1609459230000.0],[3.0,1609459260000.0],[4.0,1609459290000.0],[5.0,1609459320000.0],[4.0,1609459350000.0],[3.0,1609459380000.0],[2.0,1609459410000.0],[1.0,1609459440000.0],[0.0,1609459470000.0]]}]' ]; then
        echo "Success: test_query_endpoint"
    else
        echo "Failure: test_query_endpoint"
        TEST_GLOBAL_RESULT=1
    fi
}

test_index_endpoint
test_search_endpoint
test_query_endpoint
test_annotations_endpoint

kill "$TEST_PID"

exit $TEST_GLOBAL_RESULT
