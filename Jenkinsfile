
properties([
  parameters([
    string(
      name: 'Build_Image',
      defaultValue: 'All',
      description: 'Build image in the jenkins'
    )
  ])
])


node('ccis') {
    stage('Setup Environment') {
        cleanWs()
        checkout scm
        def image = docker.build('cureos-builder', '.')
        env.IMAGE_ID = image.id
    }
    
    stage('Build with Make') {
        docker.image(env.IMAGE_ID).inside("--user jenkins") {
            sh '''
                echo "=== Environment Check ==="
                rustc --version
                cargo --version
                make --version
                
                echo ""
                echo "=== Building with Make ==="
                make all
                
            '''
        }
    }
    
    stage('Archive') {
        archiveArtifacts artifacts: 'build/**/*.img, bin/**/*',
                        fingerprint: true
    }
}