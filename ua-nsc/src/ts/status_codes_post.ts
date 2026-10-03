
export function statusCodeIsGood(statusCode: StatusCode): boolean {
    return (statusCode & 0xf0000000) === 0x00000000;
}

export function statusCodeIsUncertain(statusCode: StatusCode): boolean {
    return (statusCode & 0xf0000000) === 0x40000000;
}

export function statusCodeIsBad(statusCode: StatusCode): boolean {
    return (statusCode & 0xf0000000) === 0x80000000;
}
