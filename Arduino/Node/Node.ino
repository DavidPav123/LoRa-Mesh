#include <SoftwareSerial.h>

// Define the RX and TX pins for the LoRa module
int rxPin = 2;  // Connect to TX of LoRa module
int txPin = 3;  // Connect to RX of LoRa module

// Initialize software serial for the LoRa module
SoftwareSerial LoRaSerial(rxPin, txPin);

void setup() {
    // Set up the software serial port for the LoRa module
    LoRaSerial.begin(9600);

    // Open serial communications for debugging
    //Serial.begin(9600);
    //Serial.println("Serial communication with LoRa module started.");
}

void loop() {
    static String inputString = "";
    static boolean stringComplete = false;

    // Read data from the LoRa module if available
    if (LoRaSerial.available()) {
        char inChar = (char)LoRaSerial.read();
        inputString += inChar;
        if (inChar == '\n') {
            stringComplete = true;
        }
    }

    // If the string is complete, process it and send it to the LoRa module
    if (stringComplete) {
        if (inputString[1] == 'R' && inputString[2] == 'C') {
            // Split the string into parts
            int index = 0;
            String parts[3] = { "", "", "" };

            for (int i = 0; i < 3; i++) {
                int nextIndex = inputString.indexOf(',', index);
                if (nextIndex == -1) nextIndex = inputString.length();
                parts[i] = inputString.substring(index, nextIndex);
                index = nextIndex + 1;
            }

            // Reassemble string
            parts[0] = "AT+SEND=0";
            String modifiedString = "";
            for (int i = 0; i < 3; i++) {
                modifiedString += parts[i];
                if (i < 2) modifiedString += ',';
            }

            modifiedString.trim(); //Remove any random whitespace
            modifiedString += "\r\n"; // Needed to propperly send AT commands to the LoRa module

            // Convert the modified string to a char array and send it to the LoRa module
            char charBuf[modifiedString.length() + 1];
            modifiedString.toCharArray(charBuf, modifiedString.length() + 1);
            LoRaSerial.write(charBuf, modifiedString.length());
        }

        inputString = "";
        stringComplete = false;
    }
}
