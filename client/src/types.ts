export type RegisterBody = {
    firstName: string,
    lastName: string | null | undefined,
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
    lastName: string | null | undefined,
    username: string,
}

export type ServerAppError = {
    message: string,
    type: string
}
