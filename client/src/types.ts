export type RegisterBody = {
    firstName: string,
    lastName: string
    username: string,
    email: string,
    password: string
}

export type LoginBody = {
    username: string;
    password: string
}

export type ServerResponse = {
    message: string
}

export type LoginResponse = {
    message: string,
    user: SessionUser
}

export type ErrorResponse = {
    errorMessage: string
}

export type SessionUser = {
    id: number,
    email: string,
    firstName: string,
    lastName: string
    username: string,
}

export type PublicUser = {
    id: number,
    username: string,
    firstName: string,
    lastName: string
}

export type Friend = PublicUser & {status: string, isSender?: boolean, color: string}

export type ServerAppError = {
    message: string,
    type: string
}

export type Message = {
    userId: number | undefined,
    content: string,
    fromAi?: boolean,
    streaming?: boolean
    querierId?: number
}
