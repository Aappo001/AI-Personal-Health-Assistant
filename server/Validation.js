// validation.js
function validateFirstName(firstName) {
    return firstName.trim() !== '';
}

function validateLastName(lastName) {
    return lastName.trim() !== '';
}

function validateUsername(username) {
    return username.length >= 3;
}

function validateEmail(email) {
    const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return re.test(String(email).toLowerCase());
}

function validatePassword(password) {
    return password.length >= 6;
}

export { 
    validateFirstName, 
    validateLastName, 
    validateUsername, 
    validateEmail, 
    validatePassword 
};
