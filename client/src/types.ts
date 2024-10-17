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
    user: PublicUserState
}

export type ErrorResponse = {
    errorMessage: string
}

export type PublicUserState = {
    id: number
    firstName: string,
    lastName: string
    username: string,
}