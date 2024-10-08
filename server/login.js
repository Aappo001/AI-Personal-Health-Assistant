import { 
    validateUsername, 
    validatePassword 
} from './Validation.js'; 

document.addEventListener("DOMContentLoaded", () => {
    const form = document.getElementById("login-form");
    const usernameInput = document.getElementById("username");
    const passwordInput = document.getElementById("password");
    const messageDiv = document.getElementById("message");

    // Validate username
    usernameInput.addEventListener("input", () => {
        if (!validateUsername(usernameInput.value)) {
            messageDiv.textContent = "Username must be at least 3 characters long.";
        } else {
            messageDiv.textContent = "";
        }
    });

    // Validate password
    passwordInput.addEventListener("input", () => {
        if (!validatePassword(passwordInput.value)) {
            messageDiv.textContent = "Password must be at least 6 characters long.";
        } else {
            messageDiv.textContent = "";
        }
    });

    form.addEventListener("submit", async (event) => {
        event.preventDefault();

        
        if (messageDiv.textContent) {
            return; 
        }

        const username = usernameInput.value;
        const password = passwordInput.value;

        const payload = { username, password };

        try {
            const response = await fetch('http://localhost:3000/api/login', { 
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload)
            });

            if (!response.ok) {
                throw new Error('Network response was not ok');
            }

            const data = await response.json();

            // Save important info locally (JWT and user settings)
            localStorage.setItem('jwt', response.headers.authorization); 
            localStorage.setItem('userSettings', JSON.stringify(data.userSettings)); 

            alert('Login successful!'); 

        } catch (error) {
            console.error('There was a problem with the fetch operation:', error);
            messageDiv.textContent = 'Login failed. Please check your username and password.';
        }
    });
});
