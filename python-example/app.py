from http_app import exports

from http_app.imports.types import (
    IncomingRequest,
    OutgoingResponse,
    ResponseOutparam,
    OutgoingBody,
    Fields,
    Ok,
)


class IncomingHandler(exports.IncomingHandler):
    def handle(self, request: IncomingRequest, response_out: ResponseOutparam):
        response = OutgoingResponse(200, Fields([("HELLO", b"WORLD")]))

        response_body = response.write()

        ResponseOutparam.set(response_out, Ok(response))

        response_stream = response_body.write()
        response_stream.blocking_write_and_flush(b"Hello from python!")
        response_stream.drop()

        OutgoingBody.finish(response_body, None)
