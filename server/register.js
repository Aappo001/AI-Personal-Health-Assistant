document.addEventListener("DOMContentLoaded", () => {
    const form = document.getElementById('registerForm');
    const firstNameInput = document.getElementById('firstName');
    const lastNameInput = document.getElementById('lastName');
    const usernameInput = document.getElementById('username');
    const emailInput = document.getElementById('email');
    const passwordInput = document.getElementById('password');
    const messageDiv = document.getElementById('message');

   
    form.addEventListener("keyup", () => {
        validateInputs();
    });

    
    form.addEventListener("submit", async (event) => {
        event.preventDefault();
        
        
        messageDiv.textContent = '';

        // Check validity 
        if (!validateInputs()) {
            return;
        }

        // Create a user object
        const user = {
            firstName: firstNameInput.value,
            lastName: lastNameInput.value,
            username: usernameInput.value,
            email: emailInput.value,
            password: passwordInput.value,
        };

        try {
            const response = await fetch('http://localhost:3000/register', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(user),
            });

            // Handle the response
            const data = await response.json();
            if (response.ok) {
                messageDiv.textContent = data.message; 
                messageDiv.style.color = 'green';
                form.reset(); 
            } else {
                messageDiv.textContent = data.message; 
                messageDiv.style.color = 'red'; 
            }
        } catch (error) {
            console.error('Error:', error);
        }
    });

    // Validate input fields
    function validateInputs() {
        let valid = true;

        
        messageDiv.textContent = '';

       
        if (!firstNameInput.value.trim()) {
            messageDiv.textContent += 'First name is required.\n';
            valid = false;
        }

        
        if (!lastNameInput.value.trim()) {
            messageDiv.textContent += 'Last name is required.\n';
            valid = false;
        }

       
        if (!usernameInput.value.trim()) {
            messageDiv.textContent += 'Username is required.\n';
            valid = false;
        }

       
        if (!validateEmail(emailInput.value)) {
            messageDiv.textContent += 'Please enter a valid email address.\n';
            valid = false;
        }

       
        if (passwordInput.value.length < 6) {
            messageDiv.textContent += 'Password must be at least 6 characters long.\n';
            valid = false;
        }

        return valid;
    }
    
    
    function validateEmail(email) {
        const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return re.test(String(email).toLowerCase());
    }
});
