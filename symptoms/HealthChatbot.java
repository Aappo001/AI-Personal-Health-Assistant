import java.util.ArrayList;
import java.util.List;
import java.util.Scanner;

public class HealthChatbot {
    public static void main(String[] args) {
        List<Symptom> symptoms = new ArrayList<>();

        // Create symptoms and add solutions
        Symptom stress = new Symptom("stress");
        stress.addSolution("mild", "Give yourself five minutes to compose.");
        stress.addSolution("medium", "Ask yourself how to relax more.");
        stress.addSolution("severe", "Drink plenty of water, eat healthy food.");
        symptoms.add(stress);

        Symptom depression = new Symptom("depression");
        depression.addSolution("mild", "Talk to your family.");
        depression.addSolution("medium", "Write solutions to cure your depression.");
        depression.addSolution("severe", "Give yourself little time to think.");
        symptoms.add(depression);

        Symptom headache = new Symptom("headache");
        headache.addSolution("mild", "Take Advil.");
        headache.addSolution("medium", "Take Tylenol and rest.");
        headache.addSolution("severe", "Consult a doctor immediately.");
        symptoms.add(headache);

        Symptom fever = new Symptom("fever");
        fever.addSolution("mild", "Drink fluids and rest.");
        fever.addSolution("medium", "Take ibuprofen and monitor temperature.");
        fever.addSolution("severe", "Seek medical attention.");
        symptoms.add(fever);

        Symptom stomachAche = new Symptom("stomach ache");
        stomachAche.addSolution("mild", "Drink ginger tea.");
        stomachAche.addSolution("medium", "Take antacids and rest.");
        stomachAche.addSolution("severe", "Consult a doctor if pain persists.");
        symptoms.add(stomachAche);

        Symptom drowsiness = new Symptom("drowsiness");
        drowsiness.addSolution("mild", "Take a small nap.");
        drowsiness.addSolution("medium", "Slowly rub your temples.");
        drowsiness.addSolution("severe", "Get a cup of coffee.");
        symptoms.add(drowsiness);

        Symptom anxiety = new Symptom("anxiety");
        anxiety.addSolution("mild", "Write your worries in a journal.");
        anxiety.addSolution("medium", "Drink a warm cup of herbal tea.");
        anxiety.addSolution("severe", "Loosen your muscles, sit down, and try to relax.");
        symptoms.add(anxiety);

        Scanner scanner = new Scanner(System.in);
        boolean isFinished = false;
        String satisfied = "";
        // Display symptom choices from the list
        while (isFinished == false) {
            System.out.println("Please choose a symptom from the following options:");
            for (Symptom symptom : symptoms) {
                System.out.println("- " + symptom.getName());
            }

            // Prompt user for symptom choice and validate input
            Symptom selectedSymptom = null;
            while (true) {
                System.out.println("Enter your symptom:");
                String symptomChoice = scanner.nextLine().toLowerCase();

                for (Symptom symptom : symptoms) {
                    if (symptom.getName().equals(symptomChoice)) {
                        selectedSymptom = symptom;
                        break;
                    }
                }

                if (selectedSymptom != null) {
                    break; // Valid symptom, break out of the loop
                } else {
                    System.out.println("Invalid symptom. Please choose one from the list.");
                }
            }

            // Display severity choices
            System.out.println("How severe is your " + selectedSymptom.getName() + "? (mild/medium/severe)");

            // Prompt user for severity level and validate input
            String severity;
            while (true) {
                severity = scanner.nextLine().toLowerCase();
                if (selectedSymptom.getSolution(severity) != null) {
                    break; // Valid severity, break out of the loop
                } else {
                    System.out.println("Invalid severity. Please enter mild, medium, or severe.");
                }
            }

            // Provide the appropriate solution
            String solution = selectedSymptom.getSolution(severity);
            System.out.println("Suggested treatment: " + solution);
            if(severity.equals(medium) || severity.equals(severe)){
                System.out.println("Would you like to write about what's bothering you? Type 'y' or 'n'");
                String write = scanner.nextLine().toLowerCase();
                if(write.equals("y")){
                    System.out.println("Write as much as you need to. There is no judgement here.");
                }
                {
                    System.out.println("Okay. That's your choice. Have a nice day.")
                }
            }
            // Capture user feedback on the solution
            System.out.println("Did the suggested treatment end up working? (Type 'yes' or 'no')");
            String helpfulSolution = scanner.nextLine().toLowerCase();

            // Check the user's response
            if (helpfulSolution.equals("yes")) {
                System.out.println("I am glad the solution helped.");
            } else {
                System.out.println("I apologize. I hope you get better soon.");
            }
            System.out.println("Do you any other symptoms you need remedies for? Answer 'yes' or 'no'");
            satisfied = scanner.nextLine().toLowerCase();
            if(satisfied.equals("yes")){
                isFinished = false;
            }
            scanner.close();
        }//finished with the symptoms
        System.out.println("Please enter what your mood is like:'happy','sad','nervous','angry'");
        String moodChoice = scanner.nextLine().toLowerCase();
        if(moodChoice.equals("happy")){
            System.out.println("That's great. Postivity everyday exciting");
        }
        else if(moodChoice.equals("sad") || moodChoice.equals("nervous")){
            System.out.println("I am sorry you feel that way. I hope you get better");
            System.out.println("Sometimes the best way to deal with this emotion is to accept how you feel.");
        }
        else if(moodChoice.equals("angry")){
            System.out.println("Breathe in, the breathe out slowly. It's okay to be angry.");
            System.out.println("Take deep breaths. Try to something relaxing.");
        }
        else{
            System.out.println("You did not enter a valid emotion");
        }
    }
}
