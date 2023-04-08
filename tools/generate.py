import os
import sys

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: ./generate.py <path/to/tests>")
        sys.exit(1)

    file_path = sys.argv[1]
    dirname = os.path.dirname(file_path)

    fd = open(file_path, 'r')
    content = fd.readlines()
    fd.close()

    cursor = 0

    while cursor < len(content):
        line = content[cursor]

        if "#test" in line:
            start_idx = line.find('(')
            end_idx = line.find(')')
            test_filename = line[start_idx+1:end_idx].lower().replace(' ',
                                                                      '_') + '.lox'
            test_path = dirname + '/' + test_filename
            # Extract test content
            cursor += 1
            test_content = b""

            while "#end" not in content[cursor]:
                test_content += content[cursor].encode('utf8')
                cursor += 1
            cursor += 1
            with open(test_path, 'wb') as fd:
                fd.write(test_content)

        else:
            cursor += 1
