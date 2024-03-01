#include <SoftwareSerial.h>

// Define the RX and TX pins for the LoRa module
int rxPin = 2; // Connect to TX of LoRa module
int txPin = 3; // Connect to RX of LoRa module

// Initialize software serial for the LoRa module
SoftwareSerial LoRaSerial(rxPin, txPin);

void setup() {
    // Set up the software serial port for the LoRa module
    LoRaSerial.begin(9600);

    // Open serial communications and wait for port to open:
    Serial.begin(9600);
    Serial.println("Serial communication with LoRa module started. Type AT commands:");
}

void loop() {
    static String inputString = "";
    static boolean stringComplete = false;

    // Read data from the computer and send it to the LoRa module character by character
    if (Serial.available()) {
        char inChar = (char)Serial.read();
        inputString += inChar;
        if (inChar == '\n') {
            stringComplete = true;
        }
    }

    // If the string is complete, convert it to a char array and send it to the LoRa module
    if (stringComplete) {
        char charBuf[inputString.length() + 1];
        inputString.toCharArray(charBuf, inputString.length() + 1);
        LoRaSerial.write(charBuf, inputString.length());

        inputString = "";
        stringComplete = false;
    }

    // If data is available from the LoRa module, send it back to the computer
    if (LoRaSerial.available()) {
        Serial.write(LoRaSerial.read());
    }
}
